use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

use crate::{
    dcs::{BitsPerPixel, Dcs, PixelFormat, SetAddressMode, SoftReset},
    error::{Error, InitError},
    models::{ili934x, Model},
    options::ModelOptions,
};

/// ILI9341 display in Rgb565 color mode.
pub struct ILI9341Rgb565;

/// ILI9341 display in Rgb666 color mode.
pub struct ILI9341Rgb666;

impl Model for ILI9341Rgb565 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

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
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(SoftReset)?,
        }

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili934x::init_common(dcs, delay, options, pf).map_err(Into::into)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        ili934x::write_pixels_rgb565(dcs, colors)
    }

    fn repeat_pixel_to_buffer(color: Self::ColorFormat, buf: &mut [u8]) -> Result<usize, Error> {
        crate::graphics::repeat_pixel_to_buffer_rgb565(color, buf)
    }
}

impl Model for ILI9341Rgb666 {
    type ColorFormat = Rgb666;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

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
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(SoftReset)?,
        }

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili934x::init_common(dcs, delay, options, pf).map_err(Into::into)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        ili934x::write_pixels_rgb666(dcs, colors)
    }

    fn repeat_pixel_to_buffer(color: Self::ColorFormat, buf: &mut [u8]) -> Result<usize, Error> {
        crate::graphics::repeat_pixel_to_buffer_rgb666(color, buf)
    }
}
