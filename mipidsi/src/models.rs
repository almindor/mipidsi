//! Display models.

use crate::{
    dcs::{Dcs, SetAddressMode, WriteMemoryStart},
    error::{Error, InitError},
    options::ModelOptions,
};
use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::{delay::DelayNs, digital::OutputPin};

// existing model implementations
mod gc9a01;
mod ili9341;
mod ili9342c;
mod ili934x;
mod ili9486;
mod st7735s;
mod st7789;
mod st7796;

pub use gc9a01::*;
pub use ili9341::*;
pub use ili9342c::*;
pub use ili9486::*;
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
    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand;

    /// Resets the display using a reset pin.
    fn hard_reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
    {
        rst.set_low().map_err(InitError::Pin)?;
        delay.delay_us(10);
        rst.set_high().map_err(InitError::Pin)?;

        Ok(())
    }

    /// Writes pixels to the display IC via the given display interface.
    ///
    /// Any pixel color format conversion is done here.
    fn write_pixels<DI, I>(&mut self, di: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>;

    /// Writes raw &[u8] buffer to the display IC via the given display interface.
    ///
    /// No pixel color format conversion, raw data is passed on directly.
    fn write_pixels_raw_u8<DI>(&mut self, dcs: &mut Dcs<DI>, raw_buf: &[u8]) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
    {
        dcs.write_command(WriteMemoryStart)?;

        let buf = DataFormat::U8(raw_buf);
        dcs.di.send_data(buf)?;
        Ok(())
    }
}
