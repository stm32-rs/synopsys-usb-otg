#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_hs_device::{Instance, RegisterBlock};
pub use super::super::peripherals::otg_hs_device::{
    DAINT, DAINTMSK, DCFG, DCTL, DEACHINT, DEACHINTMSK, DIEPCTL0, DIEPCTL1, DIEPCTL2, DIEPCTL3,
    DIEPCTL4, DIEPCTL5, DIEPDMA1, DIEPDMA2, DIEPDMA3, DIEPDMA4, DIEPDMA5, DIEPEACHMSK1, DIEPEMPMSK,
    DIEPINT0, DIEPINT1, DIEPINT2, DIEPINT3, DIEPINT4, DIEPINT5, DIEPMSK, DIEPTSIZ0, DIEPTSIZ1,
    DIEPTSIZ2, DIEPTSIZ3, DIEPTSIZ4, DIEPTSIZ5, DOEPCTL0, DOEPCTL1, DOEPCTL2, DOEPCTL3, DOEPCTL4,
    DOEPCTL5, DOEPEACHMSK1, DOEPINT0, DOEPINT1, DOEPINT2, DOEPINT3, DOEPINT4, DOEPINT5, DOEPMSK,
    DOEPTSIZ0, DOEPTSIZ1, DOEPTSIZ2, DOEPTSIZ3, DOEPTSIZ4, DOEPTSIZ5, DSTS, DTHRCTL, DTXFSTS0,
    DTXFSTS1, DTXFSTS2, DTXFSTS3, DTXFSTS4, DTXFSTS5, DVBUSDIS, DVBUSPULSE,
};

/// Access functions for the OTG_HS_DEVICE peripheral instance
pub mod OTG_HS_DEVICE {
    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x40040800,
        _marker: ::core::marker::PhantomData,
    };
}

/// Raw pointer to OTG_HS_DEVICE
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_HS_DEVICE: *const RegisterBlock = 0x40040800 as *const _;
