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

use super::Model;

/// Module containing all ST7789 variants.
mod variants;

/// ST7789 display in Rgb565 color mode.
///
/// Interfaces implemented by the [display-interface](https://crates.io/crates/display-interface) are supported.
pub struct ST7789;

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

    fn default_options() -> crate::ModelOptions {
        let mut options = ModelOptions::with_sizes((240, 320), (240, 320));
        options.set_invert_colors(ColorInversion::Normal);

        options
    }
}
