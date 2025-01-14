use embedded_graphics_core::pixelcolor::{Bgr565, Rgb565, Rgb666};
use embedded_hal::delay::DelayNs;

use crate::{
    dcs::{BitsPerPixel, PixelFormat, SetAddressMode},
    interface::Interface,
    models::{ili934x, Model},
    options::ModelOptions,
};

/// ILI9341 display in Rgb565 color mode.
pub struct ILI9341Rgb565;

/// ILI9341 display in Rgb666 color mode.
pub struct ILI9341Rgb666;

/// ILI9341 display in Bgr565 color mode.
pub struct ILI9341Bgr565;

impl Model for ILI9341Rgb565 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, DI::Error>
    where
        DELAY: DelayNs,
        DI: Interface,
    {
        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili934x::init_common(di, delay, options, pf).map_err(Into::into)
    }
}

impl Model for ILI9341Rgb666 {
    type ColorFormat = Rgb666;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, DI::Error>
    where
        DELAY: DelayNs,
        DI: Interface,
    {
        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili934x::init_common(di, delay, options, pf).map_err(Into::into)
    }
}

impl Model for ILI9341Bgr565 {
    type ColorFormat = Bgr565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, DI::Error>
    where
        DELAY: DelayNs,
        DI: Interface,
    {
        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili934x::init_common(di, delay, options, pf).map_err(Into::into)
    }
}
