//! Display models.

use crate::{
    dcs::{self, InterfaceExt, SetAddressMode},
    interface::Interface,
    options::ModelOptions,
    ConfigurationError,
};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::delay::DelayNs;

// existing model implementations
mod gc9107;
mod gc9a01;
mod ili9225;
mod ili9341;
mod ili9342c;
mod ili934x;
mod ili9486;
mod ili9488;
mod ili948x;
mod rm67162;
mod st7735s;
mod st7789;
mod st7796;

pub use gc9107::*;
pub use gc9a01::*;
pub use ili9225::*;
pub use ili9341::*;
pub use ili9342c::*;
pub use ili9486::*;
pub use ili9488::*;
pub use rm67162::*;
pub use st7735s::*;
pub use st7789::*;
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

    /// Updates the address window of the display.
    fn update_address_window<DI>(
        di: &mut DI,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), DI::Error>
    where
        DI: Interface,
    {
        di.write_command(dcs::SetColumnAddress::new(sx, ex))?;
        di.write_command(dcs::SetPageAddress::new(sy, ey))
    }

    ///
    /// Need to call [Self::wake] before issuing other commands
    ///
    fn sleep<DI, DELAY>(di: &mut DI, _delay: &mut DELAY) -> Result<(), DI::Error>
    where
        DI: Interface,
        DELAY: DelayNs,
    {
        di.write_command(dcs::EnterSleepMode)
    }
    ///
    /// Wakes the display after it's been set to sleep via [Self::sleep]
    ///
    fn wake<DI, DELAY>(di: &mut DI, _delay: &mut DELAY) -> Result<(), DI::Error>
    where
        DI: Interface,
        DELAY: DelayNs,
    {
        di.write_command(dcs::ExitSleepMode)
    }
    ///
    /// We need WriteMemoryStart befor write pixel
    ///
    fn write_memory_start<DI>(di: &mut DI) -> Result<(), DI::Error>
    where
        DI: Interface,
    {
        di.write_command(dcs::WriteMemoryStart)
    }
    ///
    /// SoftReset
    ///
    fn software_reset<DI>(di: &mut DI) -> Result<(), DI::Error>
    where
        DI: Interface,
    {
        di.write_command(dcs::SoftReset)
    }
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
