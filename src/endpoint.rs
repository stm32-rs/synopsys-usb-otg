use core::mem;
use cortex_m::interrupt::{self, Mutex, CriticalSection};
use usb_device::{Result, UsbError, UsbDirection};
use usb_device::endpoint::{EndpointType, EndpointAddress};
use crate::endpoint_memory::EndpointBuffer;
use usb_device::bus::PollResult;
use crate::ral::{endpoint_in, endpoint_out, endpoint0_out};
use stm32ral::{read_reg, write_reg, modify_reg, otg_fs_global};
use crate::target::{fifo_write, fifo_read};

/// Arbitrates access to the endpoint-specific registers and packet buffer memory.
pub struct Endpoint {
    buffer: Option<Mutex<EndpointBuffer>>,
    ep_type: Option<EndpointType>,
    max_packet_size: u16,
    address: EndpointAddress,
}

impl Endpoint {
    pub fn new(address: EndpointAddress) -> Endpoint {
        Endpoint {
            buffer: None,
            ep_type: None,
            max_packet_size: 0,
            address,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.ep_type.is_some()
    }

    pub fn set_stalled(&self, stalled: bool) {
        interrupt::free(|cs| {
            if self.is_stalled() == stalled {
                return
            }

            if self.address.is_in() {
                let ep = endpoint_in::instance(self.address.index());
                modify_reg!(endpoint_in, ep, DIEPCTL, Stall: stalled as u32);
            } else {
                let ep = endpoint_out::instance(self.address.index());
                modify_reg!(endpoint_out, ep, DOEPCTL, Stall: stalled as u32);
            }
        })
    }

    pub fn is_stalled(&self) -> bool {
        let stall = if self.address.is_in() {
            let ep = endpoint_in::instance(self.address.index());
            read_reg!(endpoint_in, ep, DIEPCTL, Stall)
        } else {
            let ep = endpoint_out::instance(self.address.index());
            read_reg!(endpoint_out, ep, DOEPCTL, Stall)
        };
        stall != 0
    }

    pub fn ep_type(&self) -> Option<EndpointType> {
        self.ep_type
    }

    pub fn set_ep_type(&mut self, ep_type: EndpointType) {
        self.ep_type = Some(ep_type);
    }

    pub fn configure(&self, cs: &CriticalSection) {
        if self.address.index() == 0 {
            let mpsiz = match self.max_packet_size {
                8 => 0b11,
                16 => 0b10,
                32 => 0b01,
                64 => 0b00,
                other => panic!("Unsupported EP0 size: {}", other),
            };

            if self.address.is_in() {
                let regs = endpoint_in::instance(self.address.index());
                write_reg!(endpoint_in, regs, DIEPCTL, MPSIZ: mpsiz as u32);
                write_reg!(endpoint_in, regs, DIEPTSIZ, PKTCNT: 0, XFRSIZ: self.max_packet_size as u32);
                modify_reg!(endpoint_in, regs, DIEPCTL, EPENA: 1, SNAK: 1);
            } else {
                let regs = endpoint0_out::instance();
                write_reg!(endpoint0_out, regs, DOEPTSIZ0, STUPCNT: 1, PKTCNT: 1, XFRSIZ: self.max_packet_size as u32);
                modify_reg!(endpoint0_out, regs, DOEPCTL0, EPENA: 1, SNAK: 1);
            }

            let global = unsafe { otg_fs_global::OTG_FS_GLOBAL::steal() };
            write_reg!(otg_fs_global, global, FS_GNPTXFSIZ,
                TX0FD: (self.max_packet_size as u32) >> 2,
                TX0FSA: 0x200
            );

        } else {
            unimplemented!()
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<()> {
        let ep = endpoint_in::instance(self.address.index());
        if read_reg!(endpoint_in, ep, DIEPCTL, USBAEP) != 0 {
            return Err(UsbError::InvalidEndpoint);
        }
        if read_reg!(endpoint_in, ep, DIEPTSIZ, PKTCNT) != 0 {
            return Err(UsbError::WouldBlock);
        }

        let ep = endpoint_in::instance(self.address.index());
        write_reg!(endpoint_in, ep, DIEPTSIZ, PKTCNT: 1, XFRSIZ: buf.len() as u32);
        modify_reg!(endpoint_in, ep, DIEPCTL, CNAK: 1, EPENA: 1);

        fifo_write(self.address.index(), buf);

        Ok(())
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let ep = endpoint_out::instance(self.address.index());
        if read_reg!(endpoint_out, ep, DOEPCTL, USBAEP) != 0 {
            return Err(UsbError::InvalidEndpoint);
        }

        let global = unsafe { otg_fs_global::OTG_FS_GLOBAL::steal() };
        let count = read_reg!(otg_fs_global, global, GRXSTSP, BCNT) as usize;

        assert!(count <= buf.len());

        fifo_read(0usize, &mut buf[..count]);

        Ok(count)
    }
}


pub struct DeviceEndpoints {
    out_ep: [Endpoint; 4],
    in_ep: [Endpoint; 4],
}

impl DeviceEndpoints {
    pub fn new() -> Self {
        let out_ep = [
            Endpoint::new(EndpointAddress::from_parts(0, UsbDirection::Out)),
            Endpoint::new(EndpointAddress::from_parts(1, UsbDirection::Out)),
            Endpoint::new(EndpointAddress::from_parts(2, UsbDirection::Out)),
            Endpoint::new(EndpointAddress::from_parts(3, UsbDirection::Out)),
        ];
        let in_ep = [
            Endpoint::new(EndpointAddress::from_parts(0, UsbDirection::In)),
            Endpoint::new(EndpointAddress::from_parts(1, UsbDirection::In)),
            Endpoint::new(EndpointAddress::from_parts(2, UsbDirection::In)),
            Endpoint::new(EndpointAddress::from_parts(3, UsbDirection::In)),
        ];
        Self {
            out_ep,
            in_ep,
        }
    }

    fn find_free(
        endpoints: &mut [Endpoint],
        ep_addr: Option<EndpointAddress>
    ) -> Result<&mut Endpoint>
    {
        if let Some(address) = ep_addr {
            for ep in endpoints {
                if ep.address == address {
                    if !ep.is_initialized() {
                        return Ok(ep);
                    } else {
                        return Err(UsbError::InvalidEndpoint);
                    }
                }
            }
            Err(UsbError::InvalidEndpoint)
        } else {
            for ep in &mut endpoints[1..] {
                if !ep.is_initialized() {
                    return Ok(ep)
                }
            }
            Err(UsbError::EndpointOverflow)
        }
    }

    pub fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        _interval: u8) -> Result<EndpointAddress>
    {
        if ep_dir == UsbDirection::Out {
            let ep = Self::find_free(&mut self.out_ep, ep_addr)?;
            ep.ep_type = Some(ep_type);
            ep.max_packet_size = max_packet_size;

            // TODO: allocate buffers

            Ok(ep.address)
        } else {
            let ep = Self::find_free(&mut self.in_ep, ep_addr)?;
            ep.ep_type = Some(ep_type);
            ep.max_packet_size = max_packet_size;

            // TODO: allocate buffers

            Ok(ep.address)
        }
    }

    pub fn write_packet(&self, ep_addr: EndpointAddress, buf: &[u8]) -> Result<()> {
        if !ep_addr.is_in() || ep_addr.index() >= 4 {
            return Err(UsbError::InvalidEndpoint);
        }

        self.in_ep[ep_addr.index()].write(buf)
    }

    pub fn read_packet(&self, ep_addr: EndpointAddress, buf: &mut [u8]) -> Result<usize> {
        if !ep_addr.is_out() || ep_addr.index() >= 4 {
            return Err(UsbError::InvalidEndpoint);
        }

        self.out_ep[ep_addr.index()].read(buf)
    }

    pub fn poll(&self) -> PollResult {
        let mut ep_out = 0;
        let mut ep_in_complete = 0;
        let mut ep_setup = 0;

        for ep in &self.in_ep {
            if ep.is_initialized() {
                let ep_regs = endpoint_in::instance(ep.address.index());
                if read_reg!(endpoint_in, ep_regs, DIEPINT, XFRC) != 0 {
                    write_reg!(endpoint_in, ep_regs, DIEPINT, XFRC: 1);
                    ep_in_complete |= (1 << ep.address.index());
                }
            }
        }

        for ep in &self.out_ep {
            if ep.is_initialized() {
                let ep_regs = endpoint_out::instance(ep.address.index());
                if read_reg!(endpoint_out, ep_regs, DOEPINT, XFRC) != 0 {
                    write_reg!(endpoint_out, ep_regs, DOEPINT, XFRC: 1);
                    ep_out |= (1 << ep.address.index());
                }
                if read_reg!(endpoint_out, ep_regs, DOEPINT, STUP) != 0 {
                    write_reg!(endpoint_out, ep_regs, DOEPINT, STUP: 1);
                    ep_setup |= (1 << ep.address.index());
                }
            }
        }

        if (ep_in_complete | ep_out | ep_setup) != 0 {
            PollResult::Data { ep_out, ep_in_complete, ep_setup }
        } else {
            PollResult::None
        }
    }

    pub fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        self.out_ep[ep_addr.index()].set_stalled(stalled)
    }

    pub fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        self.out_ep[ep_addr.index()].is_stalled()
    }

    pub fn configure_all(&self, cs: &CriticalSection) {
        for ep in &self.in_ep {
            if ep.is_initialized() {
                ep.configure(cs);
            }
        }

        for ep in &self.out_ep {
            if ep.is_initialized() {
                ep.configure(cs);
            }
        }
    }
}
