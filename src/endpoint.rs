use usb_device::{Result, UsbError, UsbDirection};
use usb_device::endpoint::{EndpointType, EndpointAddress, EndpointDescriptor};
use crate::endpoint_memory::{EndpointBuffer, EndpointBufferState};
use crate::ral::{read_reg, write_reg, modify_reg, endpoint_in, endpoint_out, endpoint0_out};
use crate::target::fifo_write;
use crate::target::interrupt::{self, CriticalSection, Mutex};
use core::ops::{Deref, DerefMut};
use core::cell::RefCell;

pub fn set_stalled(ep_addr: EndpointAddress, stalled: bool) {
    interrupt::free(|_| {
        if is_stalled(ep_addr) == stalled {
            return
        }

        match ep_addr.direction() {
            UsbDirection::Out => {
                let ep = endpoint_in::instance(ep_addr.number());
                modify_reg!(endpoint_in, ep, DIEPCTL, STALL: stalled as u32);
            },
            UsbDirection::In => {
                let ep = endpoint_out::instance(ep_addr.number());
                modify_reg!(endpoint_out, ep, DOEPCTL, STALL: stalled as u32);
            },
        }
    })
}

pub fn is_stalled(ep_addr: EndpointAddress) -> bool {
    let stall = match ep_addr.direction() {
        UsbDirection::Out => {
            let ep = endpoint_in::instance(ep_addr.number());
            read_reg!(endpoint_in, ep, DIEPCTL, STALL)
        },
        UsbDirection::In => {
            let ep = endpoint_out::instance(ep_addr.number());
            read_reg!(endpoint_out, ep, DOEPCTL, STALL)
        },
    };
    stall != 0
}

/// Arbitrates access to the endpoint-specific registers and packet buffer memory.
pub struct Endpoint {
    descriptor: EndpointDescriptor,
}

impl Endpoint {
    pub fn new(descriptor: EndpointDescriptor) -> Endpoint {
        Endpoint { descriptor }
    }

    #[inline(always)]
    pub fn number(&self) -> u8 {
        self.descriptor.address.number()
    }

    #[inline(always)]
    pub fn address(&self) -> EndpointAddress {
        self.descriptor.address
    }

    pub fn set_stalled(&self, stalled: bool) {
        set_stalled(self.address(), stalled);
    }

    pub fn is_stalled(&self) -> bool {
        is_stalled(self.address())
    }

    pub fn configure(&self, _cs: &CriticalSection) {
        if self.number() == 0 {
            let mpsiz = match self.descriptor.max_packet_size {
                8 => 0b11,
                16 => 0b10,
                32 => 0b01,
                64 => 0b00,
                other => panic!("Unsupported EP0 size: {}", other),
            };

            match self.address().direction() {
                UsbDirection::Out => {
                    let regs = endpoint0_out::instance();
                    write_reg!(endpoint0_out, regs, DOEPTSIZ0, STUPCNT: 1, PKTCNT: 1, XFRSIZ: self.descriptor.max_packet_size as u32);
                    modify_reg!(endpoint0_out, regs, DOEPCTL0, MPSIZ: mpsiz as u32, EPENA: 1, CNAK: 1);
                },
                UsbDirection::In => {
                    let regs = endpoint_in::instance(self.number());
                    write_reg!(endpoint_in, regs, DIEPCTL, MPSIZ: mpsiz as u32, SNAK: 1);
                    write_reg!(endpoint_in, regs, DIEPTSIZ, PKTCNT: 0, XFRSIZ: self.descriptor.max_packet_size as u32);
                },
            }
        } else {
            match self.address().direction() {
                UsbDirection::Out => {
                    let regs = endpoint_out::instance(self.number());
                    write_reg!(endpoint_out, regs, DOEPCTL,
                        SD0PID_SEVNFRM: 1,
                        CNAK: 1,
                        EPENA: 1,
                        USBAEP: 1,
                        EPTYP: self.descriptor.ep_type as u32,
                        MPSIZ: self.descriptor.max_packet_size as u32
                    );
                },
                UsbDirection::In => {
                    let regs = endpoint_in::instance(self.number());
                    write_reg!(endpoint_in, regs, DIEPCTL,
                        SNAK: 1,
                        USBAEP: 1,
                        EPTYP: self.descriptor.ep_type as u32,
                        SD0PID_SEVNFRM: 1,
                        TXFNUM: self.number() as u32,
                        MPSIZ: self.descriptor.max_packet_size as u32
                    );
                },
            }
        }
    }

    pub fn deconfigure(&self, _cs: &CriticalSection) {
        match self.address().direction() {
            UsbDirection::Out => {
                let regs = endpoint_out::instance(self.number());

                // deactivating endpoint
                modify_reg!(endpoint_out, regs, DOEPCTL, USBAEP: 0);

                // disabling endpoint
                if read_reg!(endpoint_out, regs, DOEPCTL, EPENA) != 0 && self.number() != 0 {
                    modify_reg!(endpoint_out, regs, DOEPCTL, EPDIS: 1)
                }

                // clean EP interrupts
                write_reg!(endpoint_out, regs, DOEPINT, 0xff);
            },
            UsbDirection::In => {
                let regs = endpoint_in::instance(self.number());

                // deactivating endpoint
                modify_reg!(endpoint_in, regs, DIEPCTL, USBAEP: 0);

                // TODO: flushing FIFO

                // disabling endpoint
                if read_reg!(endpoint_in, regs, DIEPCTL, EPENA) != 0 && self.number() != 0 {
                    modify_reg!(endpoint_in, regs, DIEPCTL, EPDIS: 1)
                }

                // clean EP interrupts
                write_reg!(endpoint_in, regs, DIEPINT, 0xff);

                // TODO: deconfiguring TX FIFO
            },
        }
    }
}

impl usb_device::endpoint::Endpoint for Endpoint {
    fn descriptor(&self) -> &EndpointDescriptor {
        &self.descriptor
    }

    fn enable(&mut self) {
        interrupt::free(|cs| {
            self.configure(cs);
        });
    }

    fn disable(&mut self) {
        interrupt::free(|cs| {
            self.deconfigure(cs);
        });
    }

    fn set_stalled(&mut self, stalled: bool) {
        self.set_stalled(stalled)
    }

    fn is_stalled(&self) -> bool {
        self.is_stalled()
    }
}

pub struct EndpointIn {
    common: Endpoint,
}

impl EndpointIn {
    pub fn new(descriptor: EndpointDescriptor) -> EndpointIn {
        EndpointIn {
            common: Endpoint::new(descriptor),
        }
    }

    pub fn fifo_size_words(&self) -> u32 {
        (self.common.descriptor.max_packet_size as u32 + 3) / 4
    }
}

impl usb_device::endpoint::Endpoint for EndpointIn {
    fn descriptor(&self) -> &EndpointDescriptor {
        self.common.descriptor()
    }

    fn enable(&mut self) {
        self.common.enable()
    }

    fn disable(&mut self) {
        self.common.disable()
    }

    fn set_stalled(&mut self, stalled: bool) {
        self.common.set_stalled(stalled)
    }

    fn is_stalled(&self) -> bool {
        self.common.is_stalled()
    }
}

impl usb_device::endpoint::EndpointIn for EndpointIn {
    fn write(&mut self, buf: &[u8]) -> Result<()> {
        let ep = endpoint_in::instance(self.number());
        if self.number() != 0 && read_reg!(endpoint_in, ep, DIEPCTL, EPENA) != 0{
            return Err(UsbError::WouldBlock);
        }

        if buf.len() > self.descriptor.max_packet_size as usize {
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

        fifo_write(self.number(), buf);

        Ok(())
    }
}

pub struct EndpointOut {
    common: Endpoint,
    pub(crate) buffer: Mutex<RefCell<EndpointBuffer>>,
}

impl EndpointOut {
    pub fn new(descriptor: EndpointDescriptor) -> EndpointOut {
        EndpointOut {
            common: Endpoint::new(descriptor),
            buffer: Mutex::new(RefCell::new(EndpointBuffer::default())),
        }
    }

    pub fn initialize(&mut self, buffer: EndpointBuffer) {
        interrupt::free(|cs| {
            self.buffer.borrow(cs).replace(buffer);
        });
    }

    pub fn buffer_state(&self) -> EndpointBufferState {
        interrupt::free(|cs| {
            self.buffer.borrow(cs).borrow().state()
        })
    }
}

impl usb_device::endpoint::Endpoint for EndpointOut {
    fn descriptor(&self) -> &EndpointDescriptor {
        &self.descriptor
    }

    fn enable(&mut self) {
        interrupt::free(|cs| {
            self.configure(cs);
        });
    }

    fn disable(&mut self) {
        interrupt::free(|cs| {
            self.deconfigure(cs);
        });
    }

    fn set_stalled(&mut self, stalled: bool) {
        self.set_stalled(stalled)
    }

    fn is_stalled(&self) -> bool {
        self.is_stalled()
    }
}

impl usb_device::endpoint::EndpointOut for EndpointOut {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        interrupt::free(|cs| {
            self.buffer.borrow(cs).borrow_mut().read_packet(buf)
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
