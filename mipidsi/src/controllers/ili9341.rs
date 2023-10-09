use core::marker::PhantomData;

use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    controllers::{ili934x, Controller},
    dcs::{BitsPerPixel, PixelFormat},
    error::InitError,
    Display, Error,
};

use super::private::Sealed;

/// ILI9341 controller.
pub struct ILI9341<C> {
    color_type: PhantomData<C>,
}

impl Controller for ILI9341<Rgb565> {
    type Color = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<DI, RST, DELAY>(
        display: &mut Display<Self, DI, RST>,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        DI: WriteOnlyDataCommand,
        RST: OutputPin,
        DELAY: DelayUs<u32>,
    {
        display.reset(delay)?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Rgb565>());
        ili934x::init_common(display, delay, pf).map_err(Into::into)
    }

    fn write_pixels<DI, RST, I>(
        display: &mut Display<Self, DI, RST>,
        colors: I,
    ) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Rgb565>,
    {
        ili934x::write_pixels_rgb565(display, colors)
    }
}

impl Controller for ILI9341<Rgb666> {
    type Color = Rgb666;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<DI, RST, DELAY>(
        display: &mut Display<Self, DI, RST>,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        DI: WriteOnlyDataCommand,
        RST: OutputPin,
        DELAY: DelayUs<u32>,
    {
        display.reset(delay)?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Rgb666>());
        ili934x::init_common(display, delay, pf).map_err(Into::into)
    }

    fn write_pixels<DI, RST, I>(
        display: &mut Display<Self, DI, RST>,
        colors: I,
    ) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Rgb666>,
    {
        ili934x::write_pixels_rgb666(display, colors)
    }
}

impl<C> Sealed for ILI9341<C> {}
