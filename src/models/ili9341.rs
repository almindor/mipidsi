use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    error::InitError,
    instruction::Instruction,
    models::{ili934x, write_command, Model},
    Builder, Error, ModelOptions,
};

/// ILI9341 display with Reset pin
/// in Rgb565 color mode
/// Backlight pin is not controlled
pub struct ILI9341Rgb565;

/// ILI9341 display with Reset pin
/// in Rgb666 color mode
/// Backlight pin is not controlled
pub struct ILI9341Rgb666;

impl Model for ILI9341Rgb565 {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<u8, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }

        ili934x::init_rgb565(di, delay, options).map_err(Into::into)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        ili934x::write_pixels_rgb565(di, colors)
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((240, 320), (240, 320))
    }
}

impl Model for ILI9341Rgb666 {
    type ColorFormat = Rgb666;

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<u8, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }

        ili934x::init_rgb666(di, delay, options).map_err(Into::into)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        ili934x::write_pixels_rgb666(di, colors)
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((240, 320), (240, 320))
    }
}

// simplified constructor for Display

impl<DI> Builder<DI, ILI9341Rgb565>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9341] as the [Model]
    /// with the default framebuffer size and display size of 240x320
    /// *WARNING* Rgb565 only works on non-SPI setups with the ILI9341!
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9341_rgb565(di: DI) -> Self {
        Self::with_model(di, ILI9341Rgb565)
    }
}

impl<DI> Builder<DI, ILI9341Rgb666>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9341] as the [Model]
    /// with the default framebuffer size and display size of 320x240
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9341_rgb666(di: DI) -> Self {
        Self::with_model(di, ILI9341Rgb666)
    }
}
