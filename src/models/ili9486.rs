use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{
    pixelcolor::{Rgb565, Rgb666},
    prelude::{IntoStorage, RgbColor},
};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{error::InitError, instruction::Instruction, Builder, Error, ModelOptions};

use super::{write_command, Model};

/// ILI9486 display in Rgb565 color mode (does *NOT* work with SPI)
/// Backlight pin is not controlled
pub struct ILI9486Rgb565;

/// ILI9486 display in Rgb666 color mode (works with SPI)
/// Backlight pin is not controlled
pub struct ILI9486Rgb666;

impl Model for ILI9486Rgb565 {
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
        delay.delay_us(120_000);

        let colmod = 0b0101_0101; // 16bit colors
        Ok(init_common(di, delay, options, colmod)?)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        di.send_data(buf)
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 480), (320, 480))
    }
}

impl Model for ILI9486Rgb666 {
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
        };

        delay.delay_us(120_000);

        let colmod = 0b0110_0110; // 18bit colors
        Ok(init_common(di, delay, options, colmod)?)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().flat_map(|c| {
            let red = c.r() << 2;
            let green = c.g() << 2;
            let blue = c.b() << 2;
            [red, green, blue]
        });

        let buf = DataFormat::U8Iter(&mut iter);
        di.send_data(buf)
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 480), (320, 480))
    }
}

// simplified constructor for Display

impl<DI> Builder<DI, ILI9486Rgb565>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9486] as the [Model]
    /// with the default framebuffer size and display size of 320x480
    /// *WARNING* Rgb565 only works on non-SPI setups with the ILI9486!
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9486_rgb565(di: DI) -> Self {
        Self::with_model(di, ILI9486Rgb565)
    }
}

impl<DI> Builder<DI, ILI9486Rgb666>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9486] as the [Model]
    /// with the default framebuffer size and display size of 320x480
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9486_rgb666(di: DI) -> Self {
        Self::with_model(di, ILI9486Rgb666)
    }
}

// common init for all color format models
fn init_common<DELAY, DI>(
    di: &mut DI,
    delay: &mut DELAY,
    options: &ModelOptions,
    colmod: u8,
) -> Result<u8, Error>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    let madctl = options.madctl();
    write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
    write_command(di, Instruction::COLMOD, &[colmod])?; // 18bit 256k colors
    write_command(di, Instruction::MADCTL, &[madctl])?; // left -> right, bottom -> top RGB
    write_command(di, Instruction::VCMOFSET, &[0x00, 0x48, 0x00, 0x48])?; //VCOM  Control 1 [00 40 00 40]
    write_command(di, Instruction::INVCO, &[0x0])?; //Inversion Control [00]
    write_command(di, options.invert_command(), &[])?; // set color inversion

    // optional gamma setup
    // write_command(di, Instruction::PGC, &[0x00, 0x2C, 0x2C, 0x0B, 0x0C, 0x04, 0x4C, 0x64, 0x36, 0x03, 0x0E, 0x01, 0x10, 0x01, 0x00])?; // Positive Gamma Control
    // write_command(di, Instruction::NGC, &[0x0F, 0x37, 0x37, 0x0C, 0x0F, 0x05, 0x50, 0x32, 0x36, 0x04, 0x0B, 0x00, 0x19, 0x14, 0x0F])?; // Negative Gamma Control

    write_command(di, Instruction::DFC, &[0b0000_0010, 0x02, 0x3B])?;
    write_command(di, Instruction::NORON, &[])?; // turn to normal mode
    write_command(di, Instruction::DISPON, &[])?; // turn on display

    // DISPON requires some time otherwise we risk SPI data issues
    delay.delay_us(120_000);

    Ok(madctl)
}
