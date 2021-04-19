#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go full speed
//!
//! Used by: stm32f401, stm32f405, stm32f407, stm32f411, stm32f412, stm32f413, stm32f427, stm32f429, stm32f446, stm32f469

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_fs_host::Instance;
pub use super::super::peripherals::otg_fs_host::RegisterBlock;
pub use super::super::peripherals::otg_fs_host::{
    HAINT, HAINTMSK, HCCHAR0, HCCHAR1, HCCHAR2, HCCHAR3, HCCHAR4, HCCHAR5, HCCHAR6, HCCHAR7, HCFG,
    HCINT0, HCINT1, HCINT2, HCINT3, HCINT4, HCINT5, HCINT6, HCINT7, HCINTMSK0, HCINTMSK1,
    HCINTMSK2, HCINTMSK3, HCINTMSK4, HCINTMSK5, HCINTMSK6, HCINTMSK7, HCTSIZ0, HCTSIZ1, HCTSIZ2,
    HCTSIZ3, HCTSIZ4, HCTSIZ5, HCTSIZ6, HCTSIZ7, HFIR, HFNUM, HPRT, HPTXSTS,
};

/// Access functions for the OTG_FS_HOST peripheral instance
pub mod OTG_FS_HOST {
    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x50000400,
        _marker: ::core::marker::PhantomData,
    };

}

/// Raw pointer to OTG_FS_HOST
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_FS_HOST: *const RegisterBlock = 0x50000400 as *const _;
