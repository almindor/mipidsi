use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{
    pixelcolor::{Rgb565, Rgb666},
    prelude::{IntoStorage, RgbColor},
};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    error::InitError, instruction::Instruction, DisplayBuilder, DisplayOptions, Error, ModelOptions,
};

use super::{write_command, Model};

/// ILI9342C display with Reset pin
/// in Rgb565 color mode
/// Backlight pin is not controlled
pub struct ILI9342CRgb565;

/// ILI9342C display with Reset pin
/// in Rgb666 color mode
/// Backlight pin is not controlled
pub struct ILI9342CRgb666;

impl Model for ILI9342CRgb565 {
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
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }

        delay.delay_us(120_000);

        write_command(di, Instruction::COLMOD, &[0b0101_0101])?; // 16bit 65k colors

        Ok(init_common(di, delay, madctl)?)
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
}

impl Model for ILI9342CRgb666 {
    type ColorFormat = Rgb666;

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
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }

        delay.delay_us(120_000);

        write_command(di, Instruction::COLMOD, &[0b0110_0110])?; // 18bit 262k colors

        Ok(init_common(di, delay, madctl)?)
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
}

// simplified constructor for Display

impl<DI> DisplayBuilder<DI, ILI9342CRgb565>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9342C] as the [Model]
    /// *WARNING* Rgb565 only works on non-SPI setups with the ILI9342C!
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9342c_rgb565(di: DI) -> Self {
        Self::new(
            di,
            ILI9342CRgb565,
            ModelOptions::with_display_size(DisplayOptions::default(), 320, 240),
        )
    }
}

impl<DI> DisplayBuilder<DI, ILI9342CRgb666>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9342C] as the [Model]
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9342c_rgb666(di: DI) -> Self {
        Self::new(
            di,
            ILI9342CRgb666,
            ModelOptions::with_display_size(DisplayOptions::default(), 320, 240),
        )
    }
}

// common init for all color format models
fn init_common<DELAY, DI>(di: &mut DI, delay: &mut DELAY, madctl: u8) -> Result<u8, Error>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    let madctl = madctl ^ 0b0000_1000; // this model has flipped RGB/BGR bit;

    write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
    write_command(di, Instruction::MADCTL, &[madctl])?; // left -> right, bottom -> top RGB
    write_command(di, Instruction::INVCO, &[0x0])?; //Inversion Control [00]

    write_command(di, Instruction::NORON, &[])?; // turn to normal mode
    write_command(di, Instruction::DISPON, &[])?; // turn on display

    // DISPON requires some time otherwise we risk SPI data issues
    delay.delay_us(120_000);

    Ok(madctl)
}
