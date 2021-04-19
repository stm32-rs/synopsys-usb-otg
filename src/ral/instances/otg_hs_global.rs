#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_hs_global::Instance;
pub use super::super::peripherals::otg_hs_global::RegisterBlock;
pub use super::super::peripherals::otg_hs_global::{
    CID, DIEPTXF1, DIEPTXF2, DIEPTXF3, DIEPTXF4, DIEPTXF5, GAHBCFG, PHYCR, GCCFG, GINTMSK, GINTSTS,
    GNPTXFSIZ, GNPTXSTS, GOTGCTL, GOTGINT, GRSTCTL, GRXFSIZ, GRXSTSP, GRXSTSR, GUSBCFG, HPTXFSIZ,
};

/// Access functions for the OTG_HS_GLOBAL peripheral instance
pub mod OTG_HS_GLOBAL {
    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x40040000,
        _marker: ::core::marker::PhantomData,
    };
}

/// Raw pointer to OTG_HS_GLOBAL
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_HS_GLOBAL: *const RegisterBlock = 0x40040000 as *const _;
