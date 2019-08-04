#![allow(non_snake_case)]

pub mod endpoint_in {
    use stm32ral::RWRegister;
    use core::marker::PhantomData;

    pub use stm32ral::stm32f4::peripherals::otg_fs_device_v1::{
        DIEPCTL1 as DIEPCTL,
        DIEPINT1 as DIEPINT,
        DIEPTSIZ1 as DIEPTSIZ,
        DTXFSTS1 as DTXFSTS,
    };

    pub struct RegisterBlock {
        pub DIEPCTL: RWRegister<u32>,
        _reserved0: u32,
        pub DIEPINT: RWRegister<u32>,
        _reserved1: u32,
        pub DIEPTSIZ: RWRegister<u32>,
        _reserved2: u32,
        pub DTXFSTS: RWRegister<u32>,
        _reserved3: u32,
    }

    pub struct Instance {
        pub(crate) addr: u32,
        pub(crate) _marker: PhantomData<*const RegisterBlock>,
    }

    impl ::core::ops::Deref for Instance {
        type Target = RegisterBlock;
        #[inline(always)]
        fn deref(&self) -> &RegisterBlock {
            unsafe { &*(self.addr as *const _) }
        }
    }

    #[inline(always)]
    pub fn instance(index: usize) -> Instance {
        let base_address = 0x5000_0000;
        Instance {
            addr: base_address + 0x900 + 0x20 * (index as u32),
            _marker: PhantomData,
        }
    }
}

pub mod endpoint0_out {
    use stm32ral::RWRegister;
    use core::marker::PhantomData;

    pub use stm32ral::stm32f4::peripherals::otg_fs_device_v1::{
        DOEPCTL0,
        DOEPINT0,
        DOEPTSIZ0,
    };

    pub struct RegisterBlock {
        pub DOEPCTL0: RWRegister<u32>,
        _reserved0: u32,
        pub DOEPINT0: RWRegister<u32>,
        _reserved1: u32,
        pub DOEPTSIZ0: RWRegister<u32>,
        _reserved2: [u32; 3],
    }

    pub struct Instance {
        pub(crate) addr: u32,
        pub(crate) _marker: PhantomData<*const RegisterBlock>,
    }

    impl ::core::ops::Deref for Instance {
        type Target = RegisterBlock;
        #[inline(always)]
        fn deref(&self) -> &RegisterBlock {
            unsafe { &*(self.addr as *const _) }
        }
    }

    #[inline(always)]
    pub fn instance() -> Instance {
        let base_address = 0x5000_0000;
        Instance {
            addr: base_address + 0xb00,
            _marker: PhantomData,
        }
    }
}

pub mod endpoint_out {
    use stm32ral::RWRegister;
    use core::marker::PhantomData;

    pub use stm32ral::stm32f4::peripherals::otg_fs_device_v1::{
        DOEPCTL1 as DOEPCTL,
        DOEPINT1 as DOEPINT,
        DOEPTSIZ1 as DOEPTSIZ,
    };

    pub struct RegisterBlock {
        pub DOEPCTL: RWRegister<u32>,
        _reserved0: u32,
        pub DOEPINT: RWRegister<u32>,
        _reserved1: u32,
        pub DOEPTSIZ: RWRegister<u32>,
        _reserved2: [u32; 3],
    }

    pub struct Instance {
        pub(crate) addr: u32,
        pub(crate) _marker: PhantomData<*const RegisterBlock>,
    }

    impl ::core::ops::Deref for Instance {
        type Target = RegisterBlock;
        #[inline(always)]
        fn deref(&self) -> &RegisterBlock {
            unsafe { &*(self.addr as *const _) }
        }
    }

    #[inline(always)]
    pub fn instance(index: usize) -> Instance {
        let base_address = 0x5000_0000;
        Instance {
            addr: base_address + 0xb00 + 0x20 * (index as u32),
            _marker: PhantomData,
        }
    }
}
