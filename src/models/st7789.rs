use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{instruction::Instruction, Display, Error};

use super::{write_command, Model};

/// ST7789 SPI display with Reset pin
/// Only SPI with DC pin interface is supported
pub struct ST7789;

impl Model for ST7789 {
    type ColorFormat = Rgb565;

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

        write_command(di, Instruction::COLMOD, &[0b0101_0101])?; // 16bit 65k colors
        write_command(di, Instruction::MADCTL, &[0b0000_0000])?; // left -> right, bottom -> top RGB
        write_command(di, Instruction::VCMOFSET, &[0x00, 0x48, 0x00, 0x48])?; //VCOM  Control 1 [00 40 00 40]
        write_command(di, Instruction::INVCO, &[0x0])?; //Inversion Control [00]

        write_command(di, Instruction::INVON, &[])?;
        // delay.delay_us(10_000);

        write_command(di, Instruction::NORON, &[])?; // turn to normal mode
        write_command(di, Instruction::DISPON, &[])?; // turn on display

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(())
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

    fn display_size(&self) -> (u16, u16) {
        (240, 240)
    }

    fn framebuffer_size(&self) -> (u16, u16) {
        (240, 320)
    }
}

// simplified constructor on Display

impl<DI, RST> Display<DI, RST, ST7789>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Creates a new [Display] instance with [ST7789] as the [Model]
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `model` - the display [Model]
    ///
    pub fn st7789(di: DI, rst: RST) -> Self {
        Self::with_model(di, rst, ST7789::new())
    }
}
