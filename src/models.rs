//! Display models.

use crate::{dcs::SetAddressMode, interface::Interface, options::ModelOptions, ConfigurationError};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::delay::DelayNs;

// existing model implementations
#[cfg(not(feature = "ili9225"))]
mod gc9107;
#[cfg(not(feature = "ili9225"))]
mod gc9a01;
#[cfg(feature = "ili9225")]
mod ili9225;
#[cfg(not(feature = "ili9225"))]
mod ili9341;
#[cfg(not(feature = "ili9225"))]
mod ili9342c;
#[cfg(not(feature = "ili9225"))]
mod ili934x;
#[cfg(not(feature = "ili9225"))]
mod ili9486;
#[cfg(not(feature = "ili9225"))]
mod ili9488;
#[cfg(not(feature = "ili9225"))]
mod ili948x;
#[cfg(not(feature = "ili9225"))]
mod rm67162;
#[cfg(not(feature = "ili9225"))]
mod st7735s;
#[cfg(not(feature = "ili9225"))]
mod st7789;
#[cfg(not(feature = "ili9225"))]
mod st7796;

#[cfg(not(feature = "ili9225"))]
pub use gc9107::*;
#[cfg(not(feature = "ili9225"))]
pub use gc9a01::*;
#[cfg(not(feature = "ili9225"))]
pub use ili9341::*;
#[cfg(feature = "ili9225")]
pub use ili9225::*;
#[cfg(not(feature = "ili9225"))]
pub use ili9342c::*;
#[cfg(not(feature = "ili9225"))]
pub use ili9486::*;
#[cfg(not(feature = "ili9225"))]
pub use ili9488::*;
#[cfg(not(feature = "ili9225"))]
pub use rm67162::*;
#[cfg(not(feature = "ili9225"))]
pub use st7735s::*;
#[cfg(not(feature = "ili9225"))]
pub use st7789::*;
#[cfg(not(feature = "ili9225"))]
pub use st7796::*;

/// Display model.
pub trait Model {
    /// The color format.
    type ColorFormat: RgbColor;

    /// The framebuffer size in pixels.
    const FRAMEBUFFER_SIZE: (u16, u16);

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, ModelInitError<DI::Error>>
    where
        DELAY: DelayNs,
        DI: Interface;
}

/// Error returned by [`Model::init`].
///
/// This error type is used internally by implementations of the [`Model`]
/// trait.
pub enum ModelInitError<DiError> {
    /// Error caused by the display interface.
    Interface(DiError),

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

#[cfg(test)]
mod tests {
    use embedded_graphics::pixelcolor::Rgb565;

    use crate::{
        Builder,
        _mock::{MockDelay, MockDisplayInterface},
        interface::InterfaceKind,
        ConfigurationError, InitError,
    };

    use super::*;

    struct OnlyOneKindModel(InterfaceKind);

    impl Model for OnlyOneKindModel {
        type ColorFormat = Rgb565;

        const FRAMEBUFFER_SIZE: (u16, u16) = (16, 16);

        fn init<DELAY, DI>(
            &mut self,
            _di: &mut DI,
            _delay: &mut DELAY,
            _options: &ModelOptions,
        ) -> Result<SetAddressMode, ModelInitError<DI::Error>>
        where
            DELAY: DelayNs,
            DI: Interface,
        {
            if DI::KIND != self.0 {
                return Err(ModelInitError::InvalidConfiguration(
                    ConfigurationError::UnsupportedInterface,
                ));
            }

            Ok(SetAddressMode::default())
        }
    }

    #[test]
    fn test_assert_interface_kind_serial() {
        Builder::new(
            OnlyOneKindModel(InterfaceKind::Serial4Line),
            MockDisplayInterface,
        )
        .init(&mut MockDelay)
        .unwrap();
    }

    #[test]
    fn test_assert_interface_kind_parallel() {
        assert!(matches!(
            Builder::new(
                OnlyOneKindModel(InterfaceKind::Parallel8Bit),
                MockDisplayInterface,
            )
            .init(&mut MockDelay),
            Err(InitError::InvalidConfiguration(
                ConfigurationError::UnsupportedInterface
            ))
        ));
    }
}
