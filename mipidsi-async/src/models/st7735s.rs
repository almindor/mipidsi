use crate::{
    dcs::{
        BitsPerPixel, Dcs, ExitSleepMode, PixelFormat, SetAddressMode, SetDisplayOn, SetInvertMode,
        SetPixelFormat, SoftReset, WriteMemoryStart,
    },
    Builder, Error, ModelOptions,
};
use display_interface::{AsyncWriteOnlyDataCommand, DataFormat};
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::IntoStorage;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayUs;
use mipidsi::{error::InitError, ColorInversion};

use super::Model;

/// ST7735s display in Rgb565 color mode.
pub struct ST7735s;

impl Model for ST7735s {
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
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay).await?,
            None => dcs.write_command(SoftReset).await?,
        }
        delay.delay_us(200_000).await;

        dcs.write_command(ExitSleepMode).await?; // turn off sleep
        delay.delay_us(120_000).await;

        dcs.write_command(SetInvertMode(options.invert_colors))
            .await?; // set color inversion
        dcs.write_raw(0xB1, &[0x05, 0x3A, 0x3A]).await?; // set frame rate
        dcs.write_raw(0xB2, &[0x05, 0x3A, 0x3A]).await?; // set frame rate
        dcs.write_raw(0xB3, &[0x05, 0x3A, 0x3A, 0x05, 0x3A, 0x3A])
            .await?; // set frame rate
        dcs.write_raw(0xB4, &[0b0000_0011]).await?; // set inversion control
        dcs.write_raw(0xC0, &[0x62, 0x02, 0x04]).await?; // set power control 1
        dcs.write_raw(0xC1, &[0xC0]).await?; // set power control 2
        dcs.write_raw(0xC2, &[0x0D, 0x00]).await?; // set power control 3
        dcs.write_raw(0xC3, &[0x8D, 0x6A]).await?; // set power control 4
        dcs.write_raw(0xC4, &[0x8D, 0xEE]).await?; // set power control 5
        dcs.write_raw(0xC5, &[0x0E]).await?; // set VCOM control 1
        dcs.write_raw(
            0xE0,
            &[
                0x10, 0x0E, 0x02, 0x03, 0x0E, 0x07, 0x02, 0x07, 0x0A, 0x12, 0x27, 0x37, 0x00, 0x0D,
                0x0E, 0x10,
            ],
        )
        .await?; // set GAMMA +Polarity characteristics
        dcs.write_raw(
            0xE1,
            &[
                0x10, 0x0E, 0x03, 0x03, 0x0F, 0x06, 0x02, 0x08, 0x0A, 0x13, 0x26, 0x36, 0x00, 0x0D,
                0x0E, 0x10,
            ],
        )
        .await?; // set GAMMA -Polarity characteristics

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf)).await?; // set interface pixel format, 16bit pixel into frame memory

        dcs.write_command(madctl).await?; // set memory data access control, Top -> Bottom, RGB, Left -> Right
        dcs.write_command(SetDisplayOn).await?; // turn on display

        Ok(madctl)
    }

    async fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: AsyncWriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart).await?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf).await?;
        Ok(())
    }

    fn default_options() -> ModelOptions {
        let mut options = ModelOptions::with_sizes((80, 160), (132, 162));
        options.set_invert_colors(ColorInversion::Inverted);

        options
    }
}

// simplified constructor on Display

impl<DI> Builder<DI, ST7735s>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Creates a new display builder for ST7735s displays in Rgb565 color mode.
    ///
    /// The default framebuffer size is 132x162 pixels and display size is 80x160 pixels.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](WriteOnlyDataCommand) for communicating with the display
    ///
    pub fn st7735s(di: DI) -> Self {
        Self::with_model(di, ST7735s)
    }
}
