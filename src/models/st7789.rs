use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::no_pin::NoPin;
use crate::{instruction::Instruction, Display, Error};
use crate::{DisplayOptions, Orientation};

use super::{write_command, Model};

/// ST7789 SPI display with Reset pin
/// Only SPI with DC pin interface is supported
pub struct ST7789(DisplayOptions);

impl Model for ST7789 {
    type ColorFormat = Rgb565;

    fn new(options: DisplayOptions) -> Self {
        Self(options)
    }

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        rst: &mut Option<RST>,
        delay: &mut DELAY,
    ) -> Result<u8, Error<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        let madctl = self.0.madctl();
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }
        delay.delay_us(150_000);

        write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
        delay.delay_us(10_000);

        write_command(di, Instruction::INVOFF, &[])?;
        write_command(di, Instruction::VSCRDER, &[0u8, 0u8, 0x14u8, 0u8, 0u8, 0u8])?;
        write_command(di, Instruction::MADCTL, &[madctl])?; // left -> right, bottom -> top RGB

        write_command(di, Instruction::COLMOD, &[0b0101_0101])?; // 16bit 65k colors
        write_command(di, Instruction::INVON, &[])?;
        delay.delay_us(10_000);
        write_command(di, Instruction::NORON, &[])?; // turn to normal mode
        delay.delay_us(10_000);
        write_command(di, Instruction::DISPON, &[])?; // turn on display

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

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
        self.0.display_size(240, 240, orientation)
    }

    fn framebuffer_size(&self, orientation: Orientation) -> (u16, u16) {
        self.0.framebuffer_size(240, 320, orientation)
    }

    fn options(&self) -> &DisplayOptions {
        &self.0
    }
}

// simplified constructor on Display

impl<DI, RST> Display<DI, RST, ST7789>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Creates a new [Display] instance with [ST7789] as the [Model] with a
    /// hard reset Pin
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `model` - the display [Model]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7789(di: DI, rst: RST, options: DisplayOptions) -> Self {
        Self::with_model(di, Some(rst), ST7789::new(options))
    }
}

impl<DI> Display<DI, NoPin, ST7789>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ST7789] as the [Model] without
    /// a hard reset Pin
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `model` - the display [Model]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7789_without_rst(di: DI, options: DisplayOptions) -> Self {
        Self::with_model(di, None, ST7789::new(options))
    }
}
