use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::no_pin::NoPin;
use crate::{instruction::Instruction, Display, Error};
use crate::{DisplayOptions, Orientation};

use super::{write_command, Model};

/// ST7735s SPI display with Reset pin
/// Only SPI with DC pin interface is supported
pub struct ST7735s;

impl Model for ST7735s {
    type ColorFormat = Rgb565;

    fn new() -> Self {
        Self
    }

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        rst: &mut Option<RST>,
        delay: &mut DELAY,
        options: DisplayOptions,
    ) -> Result<u8, Error<RST::Error>>
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

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        di.send_data(buf)
    }

    fn display_size(&self, orientation: Orientation) -> (u16, u16) {
        match orientation {
            Orientation::Portrait(_) | Orientation::PortraitInverted(_) => (80, 160),
            Orientation::Landscape(_) | Orientation::LandscapeInverted(_) => (160, 80),
        }
    }

    fn framebuffer_size(&self, orientation: Orientation) -> (u16, u16) {
        match orientation {
            Orientation::Portrait(_) | Orientation::PortraitInverted(_) => (132, 162),
            Orientation::Landscape(_) | Orientation::LandscapeInverted(_) => (162, 132),
        }
    }
}

// simplified constructor on Display

impl<DI, RST> Display<DI, RST, ST7735s>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Creates a new [Display] instance with [ST7735s] as the [Model] with a
    /// hard reset Pin
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `model` - the display [Model]
    ///
    pub fn st7735s(di: DI, rst: RST) -> Self {
        Self::with_model(di, Some(rst), ST7735s::new())
    }
}

impl<DI> Display<DI, NoPin, ST7735s>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ST7735s] as the [Model] without
    /// a hard reset Pin
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `model` - the display [Model]
    ///
    pub fn st7735s_without_rst(di: DI) -> Self {
        Self::with_model(di, None, ST7735s::new())
    }
}
