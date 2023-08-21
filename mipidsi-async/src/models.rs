//! Display models.

use crate::dcs::Dcs;
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayUs;
use mipidsi::{dcs::SetAddressMode, error::InitError, Error, ModelOptions};

// existing model implementations
// mod gc9a01;
// mod ili9341;
// mod ili9342c;
// mod ili934x;
// mod ili9486;
// mod st7735s;
// mod st7789;

// pub use gc9a01::*;
// pub use ili9341::*;
// pub use ili9342c::*;
// pub use ili9486::*;
// pub use st7735s::*;
// pub use st7789::*;

/// Display model.
pub trait Model {
    /// The color format.
    type ColorFormat: RgbColor;

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    async fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs,
        DI: AsyncWriteOnlyDataCommand;

    /// Resets the display using a reset pin.
    async fn hard_reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs,
    {
        rst.set_low().map_err(InitError::Pin)?;
        delay.delay_us(10);
        rst.set_high().map_err(InitError::Pin)?;

        Ok(())
    }

    /// Writes pixels to the display IC via the given display interface.
    ///
    /// Any pixel color format conversion is done here.
    async fn write_pixels<DI, I>(&mut self, di: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>;

    /// Creates default [ModelOptions] for this particular [Model].
    ///
    /// This serves as a "sane default". There can be additional variants which will be provided via
    /// helper constructors.
    fn default_options() -> ModelOptions;
}
