use crate::{instruction::Instruction, DisplayOptions, Error};
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

// existing model implementations
mod ili9342c;
mod ili9486;
mod st7735s;
mod st7789;

pub use ili9342c::*;
pub use ili9486::*;
pub use st7735s::*;
pub use st7789::*;

pub trait Model {
    type ColorFormat: RgbColor;

    /// Common model constructor
    fn new(options: DisplayOptions) -> Self;

    /// Initializes the display for this model
    /// and returns the value of MADCTL set by init
    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        rst: &mut Option<RST>,
        delay: &mut DELAY,
    ) -> Result<u8, Error<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand;

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

    fn options(&self) -> &DisplayOptions;
}

// helper for models
pub fn write_command<DI>(
    di: &mut DI,
    command: Instruction,
    params: &[u8],
) -> Result<(), DisplayError>
where
    DI: WriteOnlyDataCommand,
{
    di.send_commands(DataFormat::U8(&[command as u8]))?;

    if !params.is_empty() {
        di.send_data(DataFormat::U8(params))
    } else {
        Ok(())
    }
}
