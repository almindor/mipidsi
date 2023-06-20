//! Async display model

use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayUs;

use crate::dcs::{Dcs, SetAddressMode};
use crate::error::InitError;
use crate::models::DefaultModel;
use crate::{Error, ModelOptions};

/// Display model.
pub trait Model: DefaultModel {
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
        delay.delay_us(10).await;
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

    /// Writes pixels to the display IC via the given display interface.
    ///
    /// Any pixel color format conversion is done here.
    async fn write_pixels_raw<DI>(
        &mut self,
        di: &mut Dcs<DI>,
        colors: &mut [u16],
    ) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand;
}
