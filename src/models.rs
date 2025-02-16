//! Display models.

use crate::{
    dcs::SetAddressMode, init_engine::InitEngine, options::ModelOptions, ConfigurationError,
};
use embedded_graphics_core::prelude::RgbColor;

// existing model implementations
// mod gc9107;
// mod gc9a01;
// mod ili9341;
// mod ili9342c;
// mod ili934x;
// mod ili9486;
// mod ili9488;
// mod ili948x;
// mod rm67162;
// mod st7735s;
mod st7789;
// mod st7796;

// pub use gc9107::*;
// pub use gc9a01::*;
// pub use ili9341::*;
// pub use ili9342c::*;
// pub use ili9486::*;
// pub use ili9488::*;
// pub use rm67162::*;
// pub use st7735s::*;
pub use st7789::*;
// pub use st7796::*;

/// Display model.
pub trait Model {
    /// The color format.
    type ColorFormat: RgbColor;

    /// The framebuffer size in pixels.
    const FRAMEBUFFER_SIZE: (u16, u16);

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    fn init<IE>(
        &mut self,
        options: &ModelOptions,
        ie: &mut IE,
    ) -> Result<SetAddressMode, ModelInitError<IE::Error>>
    where
        IE: InitEngine;
}

/// Error returned by [`Model::init`].
///
/// This error type is used internally by implementations of the [`Model`]
/// trait.
#[derive(Debug)]
pub enum ModelInitError<DiError> {
    /// Error caused by the display interface.
    Interface(DiError),

    /// The init enine's queue, used for this Model's init, was too small
    InitEngineQueueFull,

    /// Invalid configuration error.
    ///
    /// This error is returned when the configuration passed to the builder is
    /// invalid. For example, when the combination of bit depth and interface
    /// kind isn't supported by the selected model.
    InvalidConfiguration(ConfigurationError),
}

impl<DiError> From<DiError> for ModelInitError<DiError> {
    fn from(value: DiError) -> Self {
        Self::Interface(value)
    }
}

// #[cfg(test)]
// mod tests {
//     use embedded_graphics::pixelcolor::Rgb565;

//     use crate::{
//         Builder,
//         _mock::{MockDelay, MockDisplayInterface},
//         interface::InterfaceKind,
//         ConfigurationError, InitError,
//     };

//     use super::*;

//     struct OnlyOneKindModel(InterfaceKind);

//     impl Model for OnlyOneKindModel {
//         type ColorFormat = Rgb565;

//         const FRAMEBUFFER_SIZE: (u16, u16) = (16, 16);

//         fn init<DELAY, DI>(
//             &mut self,
//             _di: &mut DI,
//             _delay: &mut DELAY,
//             _options: &ModelOptions,
//         ) -> Result<SetAddressMode, ModelInitError<DI::Error>>
//         where
//             DELAY: DelayNs,
//             DI: Interface,
//         {
//             if DI::KIND != self.0 {
//                 return Err(ModelInitError::InvalidConfiguration(
//                     ConfigurationError::UnsupportedInterface,
//                 ));
//             }

//             Ok(SetAddressMode::default())
//         }
//     }

//     #[test]
//     fn test_assert_interface_kind_serial() {
//         Builder::new(
//             OnlyOneKindModel(InterfaceKind::Serial4Line),
//             MockDisplayInterface,
//         )
//         .init(&mut MockDelay)
//         .unwrap();
//     }

//     #[test]
//     fn test_assert_interface_kind_parallel() {
//         assert!(matches!(
//             Builder::new(
//                 OnlyOneKindModel(InterfaceKind::Parallel8Bit),
//                 MockDisplayInterface,
//             )
//             .init(&mut MockDelay),
//             Err(InitError::InvalidConfiguration(
//                 ConfigurationError::UnsupportedInterface
//             ))
//         ));
//     }
// }
