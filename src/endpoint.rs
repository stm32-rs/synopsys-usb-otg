use usb_device::{Result, UsbError, UsbDirection};
use usb_device::endpoint::{EndpointType, EndpointAddress};
use crate::endpoint_memory::{EndpointBuffer, EndpointBufferState};
use crate::ral::{read_reg, write_reg, modify_reg, endpoint_in, endpoint_out, endpoint0_out};
use crate::target::fifo_write;
use crate::target::interrupt::{self, CriticalSection, Mutex};
use core::ops::{Deref, DerefMut};
use core::cell::RefCell;

pub fn set_stalled(address: EndpointAddress, stalled: bool) {
    interrupt::free(|_| {
        match address.direction() {
            UsbDirection::Out => {
                let ep = endpoint_out::instance(address.index());
                modify_reg!(endpoint_out, ep, DOEPCTL, STALL: stalled as u32);
            },
            UsbDirection::In => {
                let ep = endpoint_in::instance(address.index());
                modify_reg!(endpoint_in, ep, DIEPCTL, STALL: stalled as u32);
            },
        }
    })
}

pub fn is_stalled(address: EndpointAddress) -> bool {
    let stall = match address.direction() {
        UsbDirection::Out => {
            let ep = endpoint_out::instance(address.index());
            read_reg!(endpoint_out, ep, DOEPCTL, STALL)
        },
        UsbDirection::In => {
            let ep = endpoint_in::instance(address.index());
            read_reg!(endpoint_in, ep, DIEPCTL, STALL)
        },
    };
    stall != 0
}

/// Arbitrates access to the endpoint-specific registers and packet buffer memory.
pub struct Endpoint {
    ep_type: Option<EndpointType>,
    max_packet_size: u16,
    address: EndpointAddress,
}

impl Endpoint {
    pub fn new(address: EndpointAddress) -> Endpoint {
        Endpoint {
            ep_type: None,
            max_packet_size: 0,
            address,
        }
    }

    #[inline(always)]
    pub fn address(&self) -> EndpointAddress {
        self.address
    }

    pub fn is_initialized(&self) -> bool {
        self.ep_type.is_some()
    }

    pub fn initialize(&mut self, ep_type: EndpointType, max_packet_size: u16) {
        self.ep_type = Some(ep_type);
        self.max_packet_size = max_packet_size;
    }

    pub fn set_stalled(&self, stalled: bool) {
        if !self.is_initialized() {
            return;
        }
        set_stalled(self.address, stalled)
    }

    pub fn is_stalled(&self) -> bool {
        is_stalled(self.address)
    }

    pub fn deconfigure(&self, _cs: &CriticalSection) {
        match self.address.direction() {
            UsbDirection::Out => {
                let regs = endpoint_out::instance(self.address.index());

                // deactivating endpoint
                modify_reg!(endpoint_out, regs, DOEPCTL, USBAEP: 0);

                // disabling endpoint
                if read_reg!(endpoint_out, regs, DOEPCTL, EPENA) != 0 && self.address.index() != 0 {
                    modify_reg!(endpoint_out, regs, DOEPCTL, EPDIS: 1)
                }

                // clean EP interrupts
                write_reg!(endpoint_out, regs, DOEPINT, 0xff);
            },
            UsbDirection::In => {
                let regs = endpoint_in::instance(self.address.index());

                // deactivating endpoint
                modify_reg!(endpoint_in, regs, DIEPCTL, USBAEP: 0);

                // TODO: flushing FIFO

                // disabling endpoint
                if read_reg!(endpoint_in, regs, DIEPCTL, EPENA) != 0 && self.address.index() != 0 {
                    modify_reg!(endpoint_in, regs, DIEPCTL, EPDIS: 1)
                }

                // clean EP interrupts
                write_reg!(endpoint_in, regs, DIEPINT, 0xff);

                // TODO: deconfiguring TX FIFO
            },
        }
    }
}


pub struct EndpointIn {
    common: Endpoint,
}

impl EndpointIn {
    pub fn new(address: EndpointAddress) -> EndpointIn {
        EndpointIn {
            common: Endpoint::new(address),
        }
    }

    pub fn configure(&self, _cs: &CriticalSection) {
        if self.address.index() == 0 {
            let mpsiz = match self.max_packet_size {
                8 => 0b11,
                16 => 0b10,
                32 => 0b01,
                64 => 0b00,
                other => panic!("Unsupported EP0 size: {}", other),
            };

            let regs = endpoint_in::instance(self.address.index());
            write_reg!(endpoint_in, regs, DIEPCTL, MPSIZ: mpsiz as u32, SNAK: 1);
            write_reg!(endpoint_in, regs, DIEPTSIZ, PKTCNT: 0, XFRSIZ: self.max_packet_size as u32);
        } else {
            let regs = endpoint_in::instance(self.address.index());
            write_reg!(endpoint_in, regs, DIEPCTL,
                SNAK: 1,
                USBAEP: 1,
                EPTYP: self.ep_type.unwrap() as u32,
                SD0PID_SEVNFRM: 1,
                TXFNUM: self.address.index() as u32,
                MPSIZ: self.max_packet_size as u32
            );
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<()> {
        let ep = endpoint_in::instance(self.address.index());
        if !self.is_initialized() {
            return Err(UsbError::InvalidEndpoint);
        }
        if self.address.index() != 0 && read_reg!(endpoint_in, ep, DIEPCTL, EPENA) != 0{
            return Err(UsbError::WouldBlock);
        }

        if buf.len() > self.max_packet_size as usize {
            return Err(UsbError::BufferOverflow);
        }

        if !buf.is_empty() {
            // Check for FIFO free space
            let size_words = (buf.len() + 3) / 4;
            if size_words > read_reg!(endpoint_in, ep, DTXFSTS, INEPTFSAV) as usize {
                return Err(UsbError::WouldBlock);
            }
        }

        #[cfg(feature = "fs")]
        write_reg!(endpoint_in, ep, DIEPTSIZ, PKTCNT: 1, XFRSIZ: buf.len() as u32);
        #[cfg(feature = "hs")]
        write_reg!(endpoint_in, ep, DIEPTSIZ, MCNT: 1, PKTCNT: 1, XFRSIZ: buf.len() as u32);

        modify_reg!(endpoint_in, ep, DIEPCTL, CNAK: 1, EPENA: 1);

        fifo_write(self.address.index(), buf);

        Ok(())
    }

    pub fn fifo_size_words(&self) -> u32 {
        if self.is_initialized() {
            (self.max_packet_size as u32 + 3) / 4
        } else {
            0
        }
    }
}

pub struct EndpointOut {
    common: Endpoint,
    pub(crate) buffer: Mutex<RefCell<EndpointBuffer>>,
}

impl EndpointOut {
    pub fn new(address: EndpointAddress) -> EndpointOut {
        EndpointOut {
            common: Endpoint::new(address),
            buffer: Mutex::new(RefCell::new(EndpointBuffer::default())),
        }
    }

    pub fn initialize(&mut self, ep_type: EndpointType, max_packet_size: u16, buffer: EndpointBuffer) {
        Endpoint::initialize(self, ep_type, max_packet_size);

        interrupt::free(|cs| {
            self.buffer.borrow(cs).replace(buffer);
        });
    }

    pub fn configure(&self, _cs: &CriticalSection) {
        if self.address.index() == 0 {
            let mpsiz = match self.max_packet_size {
                8 => 0b11,
                16 => 0b10,
                32 => 0b01,
                64 => 0b00,
                other => panic!("Unsupported EP0 size: {}", other),
            };

            let regs = endpoint0_out::instance();
            write_reg!(endpoint0_out, regs, DOEPTSIZ0, STUPCNT: 1, PKTCNT: 1, XFRSIZ: self.max_packet_size as u32);
            modify_reg!(endpoint0_out, regs, DOEPCTL0, MPSIZ: mpsiz as u32, EPENA: 1, CNAK: 1);
        } else {
            let regs = endpoint_out::instance(self.address.index());
            write_reg!(endpoint_out, regs, DOEPCTL,
                SD0PID_SEVNFRM: 1,
                CNAK: 1,
                EPENA: 1,
                USBAEP: 1,
                EPTYP: self.ep_type.unwrap() as u32,
                MPSIZ: self.max_packet_size as u32
            );
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        if !self.is_initialized() {
            return Err(UsbError::InvalidEndpoint);
        }

        interrupt::free(|cs| {
            self.buffer.borrow(cs).borrow_mut().read_packet(buf)
        })
    }

    pub fn buffer_state(&self) -> EndpointBufferState {
        interrupt::free(|cs| {
            self.buffer.borrow(cs).borrow().state()
        })
    }
}


impl Deref for EndpointIn {
    type Target = Endpoint;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl DerefMut for EndpointIn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl Deref for EndpointOut {
    type Target = Endpoint;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl DerefMut for EndpointOut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}
