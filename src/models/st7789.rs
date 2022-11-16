use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{error::InitError, instruction::Instruction, Error, ModelOptions};

use super::{write_command, Model};

/// Module containing all ST7789 variants and helper constructors for [Display]
mod variants;

/// ST7789 SPI display with Reset pin
/// Only SPI with DC pin interface is supported
pub struct ST7789;

impl Model for ST7789 {
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
        let madctl = options.madctl();
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }
        delay.delay_us(150_000);

        write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
        delay.delay_us(10_000);

        write_command(di, Instruction::VSCRDEF, &[0u8, 0u8, 0x14u8, 0u8, 0u8, 0u8])?;
        write_command(di, Instruction::MADCTL, &[madctl])?; // left -> right, bottom -> top RGB
        write_command(di, options.invert_command(), &[])?; // set color inversion
        write_command(di, Instruction::COLMOD, &[0b0101_0101])?; // 16bit 65k colors
        delay.delay_us(10_000);
        write_command(di, Instruction::NORON, &[])?; // turn to normal mode
        delay.delay_us(10_000);
        write_command(di, Instruction::DISPON, &[])?; // turn on display

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        di.send_data(buf)?;
        Ok(())
    }

    fn default_options() -> crate::ModelOptions {
        ModelOptions::with_sizes((240, 320), (240, 320)).with_invert_colors(true)
    }
}
