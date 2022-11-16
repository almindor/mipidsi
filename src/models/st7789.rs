use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    dcs::{Colmod, Dcs, Madctl, Vscrdef},
    error::InitError,
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
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.sw_reset()?,
        }
        delay.delay_us(150_000);

        dcs.mode_sleep(false)?; // turn off sleep
        delay.delay_us(10_000);

        // set hw scroll area based on framebuffer size
        let vcsrdef = Vscrdef::new(0, options.framebuffer_size_max(), 0);
        dcs.write_command(&vcsrdef)?;
        let madctl = Madctl::from(options);
        dcs.write_command(&madctl)?;

        dcs.invert_colors(options.invert_colors)?;
        let colmod = Colmod::new::<Self::ColorFormat>();
        dcs.write_command(&colmod)?;
        delay.delay_us(10_000);
        dcs.mode_normal(true)?;
        delay.delay_us(10_000);
        dcs.display_on(true)?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.prep_ram_write()?;

        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }

    fn default_options() -> crate::ModelOptions {
        ModelOptions::with_sizes((240, 320), (240, 320)).with_invert_colors(true)
    }
}
