use crate::{
    dcs::{
        BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat, SoftReset, WriteMemoryStart,
    },
    error::InitError,
    Builder, Error, ModelOptions,
};
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayUs;
use mipidsi::error::InitError;

use super::Model;

/// ILI9486 display in Rgb565 color mode.
pub struct ILI9486Rgb565;

/// ILI9486 display in Rgb666 color mode.
pub struct ILI9486Rgb666;

impl Model for ILI9486Rgb565 {
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
        DELAY: DelayUs<u32>,
        DI: AsyncWriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay).await?,
            None => dcs.write_command(SoftReset).await?,
        }
        delay.delay_us(120_000).await;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        Ok(init_common(dcs, delay, options, pf).await?)
    }

    async fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart).await?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf).await
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 480), (320, 480))
    }
}

impl Model for ILI9486Rgb666 {
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
        DELAY: DelayUs<u32>,
        DI: AsyncWriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay).await?,
            None => dcs.write_command(SoftReset).await?,
        };

        delay.delay_us(120_000).await;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        Ok(init_common(dcs, delay, options, pf).await?)
    }

    async fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart).await?;
        let mut iter = colors.into_iter().flat_map(|c| {
            let red = c.r() << 2;
            let green = c.g() << 2;
            let blue = c.b() << 2;
            [red, green, blue]
        });

        let buf = DataFormat::U8Iter(&mut iter);
        dcs.di.send_data(buf).await
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 480), (320, 480))
    }
}

// simplified constructor for Display

impl<DI> Builder<DI, ILI9486Rgb565>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Creates a new display builder for an ILI9486 display in Rgb565 color mode.
    ///
    /// The default framebuffer size and display size is 320x480 pixels.
    ///
    /// # Limitations
    ///
    /// The Rgb565 color mode is not supported for displays with SPI connection.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](AsyncWriteOnlyDataCommand) for communicating with the display
    ///
    pub fn ili9486_rgb565(di: DI) -> Self {
        Self::with_model(di, ILI9486Rgb565)
    }
}

impl<DI> Builder<DI, ILI9486Rgb666>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Creates a new display builder for ILI9486 displays in Rgb666 color mode.
    ///
    /// The default framebuffer size and display size is 320x480 pixels.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](AsyncWriteOnlyDataCommand) for communicating with the display
    ///
    pub fn ili9486_rgb666(di: DI) -> Self {
        Self::with_model(di, ILI9486Rgb666)
    }
}

// common init for all color format models
fn init_common<DELAY, DI>(
    dcs: &mut Dcs<DI>,
    delay: &mut DELAY,
    options: &ModelOptions,
    pixel_format: PixelFormat,
) -> Result<SetAddressMode, Error>
where
    DELAY: DelayUs<u32>,
    DI: AsyncWriteOnlyDataCommand,
{
    let madctl = SetAddressMode::from(options);
    dcs.write_command(ExitSleepMode).await?; // turn off sleep
    dcs.write_command(SetPixelFormat::new(pixel_format)).await?; // pixel format
    dcs.write_command(madctl).await?; // left -> right, bottom -> top RGB
                                      // dcs.write_command(Instruction::VCMOFSET, &[0x00, 0x48, 0x00, 0x48]).await?; //VCOM  Control 1 [00 40 00 40]
                                      // dcs.write_command(Instruction::INVCO, &[0x0]).await?; //Inversion Control [00]
    dcs.write_command(SetInvertMode(options.invert_colors))
        .await?;

    // optional gamma setup
    // dcs.write_raw(Instruction::PGC, &[0x00, 0x2C, 0x2C, 0x0B, 0x0C, 0x04, 0x4C, 0x64, 0x36, 0x03, 0x0E, 0x01, 0x10, 0x01, 0x00]).await?; // Positive Gamma Control
    // dcs.write_raw(Instruction::NGC, &[0x0F, 0x37, 0x37, 0x0C, 0x0F, 0x05, 0x50, 0x32, 0x36, 0x04, 0x0B, 0x00, 0x19, 0x14, 0x0F]).await?; // Negative Gamma Control

    dcs.write_raw(0xB6, &[0b0000_0010, 0x02, 0x3B]).await?; // DFC
    dcs.write_command(EnterNormalMode).await?; // turn to normal mode
    dcs.write_command(SetDisplayOn).await?; // turn on display

    // DISPON requires some time otherwise we risk SPI data issues
    delay.delay_us(120_000).await;

    Ok(madctl)
}
