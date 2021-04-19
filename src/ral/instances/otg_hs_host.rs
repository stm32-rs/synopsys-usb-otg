#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446, stm32f469

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_hs_host::Instance;
pub use super::super::peripherals::otg_hs_host::RegisterBlock;
pub use super::super::peripherals::otg_hs_host::{
    HAINT, HAINTMSK, HCCHAR0, HCCHAR1, HCCHAR10, HCCHAR11, HCCHAR2, HCCHAR3, HCCHAR4, HCCHAR5,
    HCCHAR6, HCCHAR7, HCCHAR8, HCCHAR9, HCDMA0, HCDMA1, HCDMA10, HCDMA11, HCDMA2, HCDMA3, HCDMA4,
    HCDMA5, HCDMA6, HCDMA7, HCDMA8, HCDMA9, HCFG, HCINT0, HCINT1, HCINT10, HCINT11, HCINT2, HCINT3,
    HCINT4, HCINT5, HCINT6, HCINT7, HCINT8, HCINT9, HCINTMSK0, HCINTMSK1, HCINTMSK10, HCINTMSK11,
    HCINTMSK2, HCINTMSK3, HCINTMSK4, HCINTMSK5, HCINTMSK6, HCINTMSK7, HCINTMSK8, HCINTMSK9,
    HCSPLT0, HCSPLT1, HCSPLT10, HCSPLT11, HCSPLT2, HCSPLT3, HCSPLT4, HCSPLT5, HCSPLT6, HCSPLT7,
    HCSPLT8, HCSPLT9, HCTSIZ0, HCTSIZ1, HCTSIZ10, HCTSIZ11, HCTSIZ2, HCTSIZ3, HCTSIZ4, HCTSIZ5,
    HCTSIZ6, HCTSIZ7, HCTSIZ8, HCTSIZ9, HFIR, HFNUM, HPRT, HPTXSTS,
};

/// Access functions for the OTG_HS_HOST peripheral instance
pub mod OTG_HS_HOST {

    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x40040400,
        _marker: ::core::marker::PhantomData,
    };

}

/// Raw pointer to OTG_HS_HOST
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_HS_HOST: *const RegisterBlock = 0x40040400 as *const _;
