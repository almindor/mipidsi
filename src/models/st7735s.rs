use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{error::InitError, instruction::Instruction, DisplayBuilder, DisplayOptions, Error};

use super::{write_command, Model, ModelOptions};

/// ST7735s SPI display with Reset pin
/// Only SPI with DC pin interface is supported
pub struct ST7735s;

impl Model for ST7735s {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        madctl: u8,
        rst: &mut Option<RST>,
    ) -> Result<u8, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        let madctl = madctl ^ 0b0000_1000; // this model has flipped RGB/BGR bit

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }
        delay.delay_us(200_000);

        write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
        delay.delay_us(120_000);

        write_command(di, Instruction::INVON, &[])?; // turn inversion on
        write_command(di, Instruction::INVON, &[])?; // turn inversion on
        write_command(di, Instruction::FRMCTR1, &[0x05, 0x3A, 0x3A])?; // set frame rate
        write_command(di, Instruction::FRMCTR2, &[0x05, 0x3A, 0x3A])?; // set frame rate
        write_command(
            di,
            Instruction::FRMCTR3,
            &[0x05, 0x3A, 0x3A, 0x05, 0x3A, 0x3A],
        )?; // set frame rate
        write_command(di, Instruction::INVCO, &[0b0000_0011])?; // set inversion control
        write_command(di, Instruction::PWR1, &[0x62, 0x02, 0x04])?; // set power control 1
        write_command(di, Instruction::PWR2, &[0xC0])?; // set power control 2
        write_command(di, Instruction::PWR3, &[0x0D, 0x00])?; // set power control 3
        write_command(di, Instruction::PWR4, &[0x8D, 0x6A])?; // set power control 4
        write_command(di, Instruction::PWR5, &[0x8D, 0xEE])?; // set power control 5
        write_command(di, Instruction::VCMOFSET, &[0x0E])?; // set VCOM control 1
        write_command(
            di,
            Instruction::PGC,
            &[
                0x10, 0x0E, 0x02, 0x03, 0x0E, 0x07, 0x02, 0x07, 0x0A, 0x12, 0x27, 0x37, 0x00, 0x0D,
                0x0E, 0x10,
            ],
        )?; // set GAMMA +Polarity characteristics
        write_command(
            di,
            Instruction::NGC,
            &[
                0x10, 0x0E, 0x03, 0x03, 0x0F, 0x06, 0x02, 0x08, 0x0A, 0x13, 0x26, 0x36, 0x00, 0x0D,
                0x0E, 0x10,
            ],
        )?; // set GAMMA -Polarity characteristics
        write_command(di, Instruction::COLMOD, &[0b0101_0101])?; // set interface pixel format, 16bit pixel into frame memory
        write_command(di, Instruction::MADCTL, &[madctl])?; // set memory data access control, Top -> Bottom, RGB, Left -> Right
        write_command(di, Instruction::DISPON, &[])?; // turn on display

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
}

// simplified constructor on Display

impl<DI> DisplayBuilder<DI, ST7735s>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ST7735s] as the [Model] with a
    /// hard reset Pin
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7735s(di: DI) -> Self {
        Self::new(
            di,
            ST7735s,
            ModelOptions::with_sizes(DisplayOptions::default(), (80, 160), (132, 162)),
        )
    }
}
