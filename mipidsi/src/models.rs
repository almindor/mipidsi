//! Display models.

use crate::{
    dcs::{Dcs, SetAddressMode},
    error::{Error, InitError},
    options::ModelOptions,
};
use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::{raw::ToBytes, Rgb565, Rgb666};
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

    /// Writes the same pixel to the given [u8] buffer. Returns byte size of the raw pixel
    /// data or 0 if not implemented yet.
    fn repeat_pixel_to_buffer(_color: Self::ColorFormat, _buf: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

// optimization helpers

fn repeat_pixel_to_buffer_rgb565(color: Rgb565, buf: &mut [u8]) -> Result<usize, Error> {
    let bytes = color.to_be_bytes();

    repeat_pixel_to_buffer_bytes(&bytes, buf)
}

fn repeat_pixel_to_buffer_rgb666(color: Rgb666, buf: &mut [u8]) -> Result<usize, Error> {
    let bytes = color.to_be_bytes();

    repeat_pixel_to_buffer_bytes(&bytes, buf)
}

fn repeat_pixel_to_buffer_bytes(bytes: &[u8], buf: &mut [u8]) -> Result<usize, Error> {
    let mut j = 0;
    for val in buf {
        *val = bytes[j];

        j += 1;
        if j >= bytes.len() {
            j = 0;
        }
    }

    Ok(bytes.len())
}
