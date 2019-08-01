use core::marker::PhantomData;
use usb_device::{Result, UsbDirection};
use usb_device::bus::{UsbBusAllocator, PollResult};
use usb_device::endpoint::{EndpointType, EndpointAddress};
use cortex_m::interrupt::{self, Mutex};
use stm32ral::{read_reg, write_reg, modify_reg, otg_fs_global, otg_fs_device, otg_fs_pwrclk};
use crate::sprintln;

use crate::target::{USB, apb_usb_enable, UsbRegisters, UsbPins};
use crate::endpoint::DeviceEndpoints;
use crate::endpoint_memory::EndpointMemoryAllocator;

const RX_FIFO_SIZE: u32 = 32;


/// USB peripheral driver for STM32 microcontrollers.
pub struct UsbBus<PINS> {
    regs: Mutex<UsbRegisters>,
    endpoints: DeviceEndpoints,
    ep_allocator: EndpointMemoryAllocator,
    pins: PhantomData<PINS>,
}

impl<PINS: UsbPins+Sync> UsbBus<PINS> {
    /// Constructs a new USB peripheral driver.
    pub fn new(regs: USB, _pins: PINS, ep_memory: &'static mut [u32]) -> UsbBusAllocator<Self> {
        let bus = UsbBus {
            regs: Mutex::new(UsbRegisters::new(regs)),
            ep_allocator: EndpointMemoryAllocator::new(ep_memory),
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
        //sprintln!("enable()");

        // Enable USB_OTG in RCC
        apb_usb_enable();

        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            //modify_reg!(otg_fs_global, regs.global, FS_GUSBCFG, PHYSEL: 1);

            // Wait for AHB ready
            while read_reg!(otg_fs_global, regs.global, FS_GRSTCTL, AHBIDL) == 0 {}

            // Configure OTG as device
            modify_reg!(otg_fs_global, regs.global, FS_GUSBCFG,
                SRPCAP: 0, // SRP capability is not enabled
                TRDT: 0x6, // ??? USB turnaround time
                FDMOD: 1 // Force device mode
            );

            // Configuring Vbus sense and SOF output
            //write_reg!(otg_fs_global, regs.global, FS_GCCFG, VBUSBSEN: 1);
            write_reg!(otg_fs_global, regs.global, FS_GCCFG, 1 << 21); // set NOVBUSSENS

            // Enable PHY clock
            write_reg!(otg_fs_pwrclk, regs.pwrclk, FS_PCGCCTL, 0);

            // Soft disconnect device
            modify_reg!(otg_fs_device, regs.device, FS_DCTL, SDIS: 1);

            // Setup USB FS speed [and frame interval]
            modify_reg!(otg_fs_device, regs.device, FS_DCFG,
                DSPD: 0b11 // Device speed: Full speed
            );

            // Setting max RX FIFO size
            write_reg!(otg_fs_global, regs.global, FS_GRXFSIZ, RX_FIFO_SIZE);

            // setting up EP0 TX FIFO SZ as 64 byte
            write_reg!(otg_fs_global, regs.global, FS_GNPTXFSIZ,
                TX0FD: 16,
                TX0FSA: RX_FIFO_SIZE
            );

            // unmask EP interrupts
            write_reg!(otg_fs_device, regs.device, DIEPMSK, XFRCM: 1);

            // unmask core interrupts
            write_reg!(otg_fs_global, regs.global, FS_GINTMSK,
                USBRST: 1, ENUMDNEM: 1,
                USBSUSPM: 1, WUIM: 1,
                IEPINT: 1, RXFLVLM: 1
            );

            // clear pending interrupts
            write_reg!(otg_fs_global, regs.global, FS_GINTSTS, 0xffffffff);

            // unmask global interrupt
            modify_reg!(otg_fs_global, regs.global, FS_GAHBCFG, GINT: 1);

            // connect(true)
            modify_reg!(otg_fs_global, regs.global, FS_GCCFG, PWRDWN: 1);
            modify_reg!(otg_fs_device, regs.device, FS_DCTL, SDIS: 0);
        });
    }

    fn reset(&self) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            self.endpoints.configure_all(cs);

            modify_reg!(otg_fs_device, regs.device, FS_DCFG, DAD: 0);
        });
    }

    fn set_device_address(&self, addr: u8) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            modify_reg!(otg_fs_device, regs.device, FS_DCFG, DAD: addr as u32);
        });
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

    fn poll(&self) -> PollResult {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            let (wakeup, suspend, enum_done, reset, iep, rxflvl) = read_reg!(otg_fs_global, regs.global, FS_GINTSTS,
                WKUPINT, USBSUSP, ENUMDNE, USBRST, IEPINT, RXFLVL
            );

            if reset != 0 {
                write_reg!(otg_fs_global, regs.global, FS_GINTSTS, USBRST: 1);

                self.endpoints.deconfigure_all(cs);

                // Flush RX
                modify_reg!(otg_fs_global, regs.global, FS_GRSTCTL, RXFFLSH: 1);
                while read_reg!(otg_fs_global, regs.global, FS_GRSTCTL, RXFFLSH) == 1 {}
            }

            if enum_done != 0 {
                write_reg!(otg_fs_global, regs.global, FS_GINTSTS, ENUMDNE: 1);

                PollResult::Reset
            } else if wakeup != 0 {
                // Clear the interrupt
                write_reg!(otg_fs_global, regs.global, FS_GINTSTS, WKUPINT: 1);

                PollResult::Resume
            } else if suspend != 0 {
                write_reg!(otg_fs_global, regs.global, FS_GINTSTS, USBSUSP: 1);

                PollResult::Suspend
            } else if (iep | rxflvl) != 0 {
                // Flags are read-only, there is no need to clear them

                self.endpoints.poll(iep != 0, rxflvl != 0)
            } else {
                PollResult::None
            }
        })
    }
}
