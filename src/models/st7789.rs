use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{delay::DelayUs, digital::OutputPin};

use crate::{
    dcs::{
        BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat, SetScrollArea, SoftReset, WriteMemoryStart,
    },
    error::InitError,
    ColorInversion, Error, ModelOptions,
};

use super::{DefaultModel, Model};

/// Module containing all ST7789 variants.
mod variants;

/// ST7789 display in Rgb565 color mode.
///
/// Interfaces implemented by the [display-interface](https://crates.io/crates/display-interface) are supported.
pub struct ST7789;

impl DefaultModel for ST7789 {
    fn default_options() -> crate::ModelOptions {
        let mut options = ModelOptions::with_sizes((240, 320), (240, 320));
        options.set_invert_colors(ColorInversion::Normal);

        options
    }
}

impl Model for ST7789 {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs,
        DI: WriteOnlyDataCommand,
    {
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(SoftReset)?,
        }
        delay.delay_us(150_000);

        dcs.write_command(ExitSleepMode)?;
        delay.delay_us(10_000);

        // set hw scroll area based on framebuffer size
        dcs.write_command(SetScrollArea::from(options))?;
        dcs.write_command(madctl)?;

        dcs.write_command(SetInvertMode(options.invert_colors))?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf))?;
        delay.delay_us(10_000);
        dcs.write_command(EnterNormalMode)?;
        delay.delay_us(10_000);
        dcs.write_command(SetDisplayOn)?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;

        let mut iter = colors.into_iter().map(Rgb565::into_storage);

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }

    fn write_pixels_raw<DI>(&mut self, dcs: &mut Dcs<DI>, colors: &mut [u16]) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
    {
        dcs.write_command(WriteMemoryStart)?;
        let buf = DataFormat::U16BE(colors);
        dcs.di.send_data(buf)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
mod asynch {
    use display_interface::{AsyncWriteOnlyDataCommand, DataFormat};
    use embedded_graphics_core::pixelcolor::IntoStorage;
    use embedded_graphics_core::pixelcolor::Rgb565;
    use embedded_hal::digital::OutputPin;
    use embedded_hal_async::delay::DelayUs;

    use crate::{
        asynch::models::Model,
        dcs::{
            BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
            SetDisplayOn, SetInvertMode, SetPixelFormat, SetScrollArea, SoftReset,
            WriteMemoryStart,
        },
        error::InitError,
        Error, ModelOptions,
    };

    use super::ST7789;

    impl Model for ST7789 {
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
                None => dcs.async_write_command(SoftReset).await?,
            }
            delay.delay_us(150_000).await;

            dcs.async_write_command(ExitSleepMode).await?;
            delay.delay_us(10_000).await;

            // set hw scroll area based on framebuffer size
            dcs.async_write_command(SetScrollArea::from(options))
                .await?;
            dcs.async_write_command(madctl).await?;

            dcs.async_write_command(SetInvertMode(options.invert_colors))
                .await?;

            let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
            dcs.async_write_command(SetPixelFormat::new(pf)).await?;
            delay.delay_us(10_000).await;
            dcs.async_write_command(EnterNormalMode).await?;
            delay.delay_us(10_000).await;
            dcs.async_write_command(SetDisplayOn).await?;

            // DISPON requires some time otherwise we risk SPI data issues
            delay.delay_us(120_000).await;

            Ok(madctl)
        }

        async fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
        where
            DI: AsyncWriteOnlyDataCommand,
            I: IntoIterator<Item = Self::ColorFormat>,
        {
            dcs.async_write_command(WriteMemoryStart).await?;

            let mut iter = colors.into_iter().map(Rgb565::into_storage);

            let buf = DataFormat::U16BEIter(&mut iter);
            dcs.di.send_data(buf).await?;
            Ok(())
        }

        async fn write_pixels_raw<DI>(
            &mut self,
            dcs: &mut Dcs<DI>,
            colors: &mut [u16],
        ) -> Result<(), Error>
        where
            DI: AsyncWriteOnlyDataCommand,
        {
            dcs.async_write_command(WriteMemoryStart).await?;
            let buf = DataFormat::U16BE(colors);
            dcs.di.send_data(buf).await?;
            Ok(())
        }
    }
}
