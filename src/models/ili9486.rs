use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::{
    pixelcolor::Rgb666,
    prelude::{IntoStorage, RgbColor},
};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{instruction::Instruction, Display, Error};

use super::{write_command, Model};

/// ILI9486 SPI display with Reset pin
/// Backlight pin is not controlled
/// Only SPI with DC pin interface is supported
pub struct ILI9486;

impl Model for ILI9486 {
    type PixelFormat = Rgb666;

    fn new() -> Self {
        Self
    }

    fn init<RST, DELAY>(
        &mut self,
        di: &mut dyn WriteOnlyDataCommand,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), Error<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
    {
        self.hard_reset(rst, delay)?;
        delay.delay_us(120_000);

        write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
        delay.delay_us(120_000);

        write_command(di, Instruction::COLMOD, &[0b0110_0110])?; // 18bit 256k colors
        write_command(di, Instruction::MADCTL, &[0b0000_0000])?; // left -> right, bottom -> top RGB
        write_command(di, Instruction::VCMOFSET, &[0x00, 0x48, 0x00, 0x48])?; //VCOM  Control 1 [00 40 00 40]
        write_command(di, Instruction::INVCO, &[0x0])?; //Inversion Control [00]

        // optional gamma setup
        // write_command(di, Instruction::PGC, &[0x00, 0x2C, 0x2C, 0x0B, 0x0C, 0x04, 0x4C, 0x64, 0x36, 0x03, 0x0E, 0x01, 0x10, 0x01, 0x00])?; // Positive Gamma Control
        // write_command(di, Instruction::NGC, &[0x0F, 0x37, 0x37, 0x0C, 0x0F, 0x05, 0x50, 0x32, 0x36, 0x04, 0x0B, 0x00, 0x19, 0x14, 0x0F])?; // Negative Gamma Control

        write_command(di, Instruction::DFC, &[0b0000_0010, 0x02, 0x3B])?;

        write_command(di, Instruction::NORON, &[])?; // turn to normal mode
        write_command(di, Instruction::DISPON, &[])?; // turn on display

        Ok(())
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::PixelFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().flat_map(|c| {
            let s = c.into_storage(); // bit-packed 18bits
                                      // we need to un-pack and pad with 2 bits each into 3 bytes of 6bit info
            let red = ((s & 0x3F) as u8) << 2;
            let green = (((s >> 6) & 0x3F) as u8) << 2;
            let blue = (((s >> 12) & 0x3F) as u8) << 2;
            [red, green, blue]
        });

        let buf = DataFormat::U8Iter(&mut iter);
        di.send_data(buf)
    }

    fn display_size(&self) -> (u16, u16) {
        (320, 480)
    }
}

// simplified constructor on Display

impl<DI, RST> Display<DI, RST, ILI9486>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Creates a new [Display] instance with [ILI9486] as the [Model]
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `model` - the display [Model]
    ///
    pub fn ili9486(di: DI, rst: RST) -> Self {
        Self::with_model(di, rst, ILI9486::new())
    }
}
