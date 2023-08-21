use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayUs;
use mipidsi::error::InitError;

use crate::{
    dcs::{BitsPerPixel, Dcs, PixelFormat, SetAddressMode, SoftReset},
    models::{ili934x, Model},
    Builder, Error, ModelOptions,
};

/// ILI9342C display in Rgb565 color mode.
pub struct ILI9342CRgb565;

/// ILI9342C display in Rgb666 color mode.
pub struct ILI9342CRgb666;

impl Model for ILI9342CRgb565 {
    type ColorFormat = Rgb565;

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
        DI: AsyncWriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay).await?,
            None => dcs.write_command(SoftReset).await?,
        }

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili934x::init_common(dcs, delay, options, pf)
            .await
            .map_err(Into::into)
    }

    async fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        ili934x::write_pixels_rgb565(dcs, colors).await
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 240), (320, 240))
    }
}

impl Model for ILI9342CRgb666 {
    type ColorFormat = Rgb666;

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
        DI: AsyncWriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay).await?,
            None => dcs.write_command(SoftReset).await?,
        }

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili934x::init_common(dcs, delay, options, pf)
            .await
            .map_err(Into::into)
    }

    async fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        ili934x::write_pixels_rgb666(dcs, colors).await
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 240), (320, 240))
    }
}

// simplified constructor for Display

impl<DI> Builder<DI, ILI9342CRgb565>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Creates a new display builder for an ILI9342C display in Rgb565 color mode.
    ///
    /// The default framebuffer size and display size is 320x240 pixels.
    ///
    /// # Limitations
    ///
    /// The Rgb565 color mode is not supported for displays with SPI connection.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](AsyncWriteOnlyDataCommand) for communicating with the display
    ///
    pub fn ili9342c_rgb565(di: DI) -> Self {
        Self::with_model(di, ILI9342CRgb565)
    }
}

impl<DI> Builder<DI, ILI9342CRgb666>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Creates a new display builder for an ILI9342C display in Rgb666 color mode.
    ///
    /// The default framebuffer size and display size is 320x240
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](AsyncWriteOnlyDataCommand) for communicating with the display
    ///
    pub fn ili9342c_rgb666(di: DI) -> Self {
        Self::with_model(di, ILI9342CRgb666)
    }
}