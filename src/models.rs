use crate::{instruction::Instruction, Error};
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

// existing model implementations
mod ili9486;
mod st7789;

pub use ili9486::*;
pub use st7789::*;

pub trait Model {
    type ColorFormat: RgbColor;

    /// Common model constructor
    fn new() -> Self;

    /// Initializes the display for this model
    fn init<RST, DELAY>(
        &mut self,
        di: &mut dyn WriteOnlyDataCommand,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), Error<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>;

    fn hard_reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), Error<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
    {
        rst.set_low().map_err(Error::Pin)?;
        delay.delay_us(10);
        rst.set_high().map_err(Error::Pin)?;

        Ok(())
    }

    /// Writes pixels to the display IC via the given DisplayInterface
    /// Any pixel color format conversion is done here
    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>;

    /// Size of the visible display as `(width, height)`
    fn display_size(&self) -> (u16, u16);

    /// Size of the display framebuffer as `(width, height)`
    fn framebuffer_size(&self) -> (u16, u16) {
        self.display_size()
    }
}

// helper for models
fn write_command(
    di: &mut dyn WriteOnlyDataCommand,
    command: Instruction,
    params: &[u8],
) -> Result<(), DisplayError> {
    di.send_commands(DataFormat::U8(&[command as u8]))?;

    if !params.is_empty() {
        di.send_data(DataFormat::U8(params))
    } else {
        Ok(())
    }
}
