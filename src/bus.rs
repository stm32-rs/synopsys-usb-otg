use core::marker::PhantomData;
use core::mem;
use usb_device::{Result, UsbDirection, UsbError};
use usb_device::bus::{UsbBusAllocator, PollResult};
use usb_device::endpoint::{EndpointType, EndpointAddress};
use cortex_m::interrupt::{self, Mutex};
use stm32ral::{read_reg, write_reg, modify_reg, otg_fs_global, otg_fs_device};

use crate::target::{USB, apb_usb_enable, delay, NUM_ENDPOINTS, UsbRegisters, UsbPins};
use crate::endpoint::{Endpoint, EndpointStatus, calculate_count_rx, EndpointIndex, DeviceEndpoints};
use crate::endpoint_memory::EndpointMemoryAllocator;


/// USB peripheral driver for STM32 microcontrollers.
pub struct UsbBus<PINS> {
    regs: Mutex<UsbRegisters>,
    endpoints: DeviceEndpoints,
    ep_allocator: EndpointMemoryAllocator,
    max_endpoint: usize,
    pins: PhantomData<PINS>,
}

impl<PINS: UsbPins+Sync> UsbBus<PINS> {
    /// Constructs a new USB peripheral driver.
    pub fn new(regs: USB, _pins: PINS, ep_memory: &'static mut [u32]) -> UsbBusAllocator<Self> {
        apb_usb_enable();

        let bus = UsbBus {
            regs: Mutex::new(UsbRegisters::new(regs)),
            ep_allocator: EndpointMemoryAllocator::new(ep_memory),
            max_endpoint: 0,
            endpoints: DeviceEndpoints::new(),
            pins: PhantomData,
        };

        UsbBusAllocator::new(bus)
    }
}

impl<PINS: Send+Sync> usb_device::bus::UsbBus for UsbBus<PINS> {
    fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8) -> Result<EndpointAddress>
    {
        self.endpoints.alloc_ep(ep_dir, ep_addr, ep_type, max_packet_size, interval)
    }

    fn enable(&mut self) {
        /*let mut max = 0;
        for (index, ep) in self.endpoints.iter().enumerate() {
            if ep.is_out_buf_set() || ep.is_in_buf_set() {
                max = index;
            }
        }

        self.max_endpoint = max;

        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            regs.cntr.modify(|_, w| w.pdwn().clear_bit());

            // There is a chip specific startup delay. For STM32F103xx it's 1µs and this should wait for
            // at least that long.
            delay(72);

            regs.btable.modify(|_, w| w.btable().bits(0));
            regs.cntr.modify(|_, w| w
                .fres().clear_bit()
                .resetm().set_bit()
                .suspm().set_bit()
                .wkupm().set_bit()
                .ctrm().set_bit());
            regs.istr.modify(|_, w| unsafe { w.bits(0) });

            #[cfg(feature = "dp_pull_up_support")]
            regs.bcdr.modify(|_, w| w.dppu().set_bit());
        });*/
    }

    fn reset(&self) {
//        interrupt::free(|cs| {
//            let regs = self.regs.borrow(cs);
//
//            regs.istr.modify(|_, w| unsafe { w.bits(0) });
//            regs.daddr.modify(|_, w| w.ef().set_bit().add().bits(0));
//
//            for ep in self.endpoints.iter() {
//                ep.configure(cs);
//            }
//        });
    }

    fn set_device_address(&self, addr: u8) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            modify_reg!(otg_fs_device, regs.device, FS_DCFG, DAD: addr as u32);
        });
    }

    fn poll(&self) -> PollResult {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            let (wkup, susp, reset, oep, iep) = read_reg!(otg_fs_global, regs.global, FS_GINTSTS,
                WKUPINT, USBSUSP, USBRST, OEPINT, IEPINT
            );

            if wkup != 0 {
                // Clear the interrupt
                write_reg!(otg_fs_global, regs.global, FS_GINTSTS, WKUPINT: 1);

                PollResult::Resume
            } else if reset != 0 {
                write_reg!(otg_fs_global, regs.global, FS_GINTSTS, USBRST: 1);

                PollResult::Reset
            } else if susp != 0 {
                write_reg!(otg_fs_global, regs.global, FS_GINTSTS, USBSUSP: 1);

                PollResult::Suspend
            } else if (oep | iep) != 0 {
                self.endpoints.poll()
            } else {
                PollResult::None
            }
        })
    }

    fn write(&self, ep_addr: EndpointAddress, buf: &[u8]) -> Result<usize> {
        self.endpoints.write_packet(ep_addr, buf).map(|_| buf.len())
    }

    fn read(&self, ep_addr: EndpointAddress, buf: &mut [u8]) -> Result<usize> {
        self.endpoints.read_packet(ep_addr, buf)
    }

    fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        self.endpoints.set_stalled(ep_addr, stalled);
    }

    fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        self.endpoints.is_stalled(ep_addr)
    }

    fn suspend(&self) {
        // Nothing to do here?
    }

    fn resume(&self) {
        // Nothing to do here?
    }
}
