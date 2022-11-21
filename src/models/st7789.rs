use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    dcs::{Colmod, Dcs, Madctl, Vscrdef},
    error::InitError,
    instruction::Instruction,
    Error, ModelOptions,
};

use super::Model;

/// Module containing all ST7789 variants and helper constructors for [Display]
mod variants;

/// ST7789 SPI display with Reset pin
/// Only SPI with DC pin interface is supported
pub struct ST7789;

impl Model for ST7789 {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<Madctl, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        let madctl = Madctl::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(Instruction::SWRESET.to_command())?,
        }
        delay.delay_us(150_000);

        dcs.write_command(Instruction::SLPOUT.to_command())?;
        delay.delay_us(10_000);

        // set hw scroll area based on framebuffer size
        dcs.write_command(Vscrdef::from(options))?;
        dcs.write_command(madctl)?;

        dcs.write_command(options.invert_colors)?;
        dcs.write_command(Colmod::new::<Self::ColorFormat>())?;
        delay.delay_us(10_000);
        dcs.write_command(Instruction::NORON.to_command())?;
        delay.delay_us(10_000);
        dcs.write_command(Instruction::DISPON.to_command())?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(Instruction::RAMWR.to_command())?;

        let mut iter = colors.into_iter().map(Rgb565::into_storage);

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }

    fn default_options() -> crate::ModelOptions {
        ModelOptions::with_sizes((240, 320), (240, 320)).with_invert_colors(true)
    }
}
