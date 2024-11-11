use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::delay::DelayNs;

use crate::{
    dcs::{BitsPerPixel, Dcs, PixelFormat, SetAddressMode},
    error::Error,
    models::{ili934x, Model},
    options::ModelOptions,
};

/// ILI9342C display in Rgb565 color mode.
pub struct ILI9342CRgb565;

/// ILI9342C display in Rgb666 color mode.
pub struct ILI9342CRgb666;

impl Model for ILI9342CRgb565 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 240);

    fn init<DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, Error>
    where
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
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

impl Model for ILI9342CRgb666 {
    type ColorFormat = Rgb666;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 240);

    fn init<DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, Error>
    where
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
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
