use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::prelude::*;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;

use crate::{error::InitError, Display, Error};

mod ili9341;
mod ili934x;

pub use ili9341::ILI9341;

pub enum NoResetPin {}

impl OutputPin for NoResetPin {
    type Error = core::convert::Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub trait Controller: Sized + private::Sealed {
    type Color: PixelColor;

    const FRAMEBUFFER_SIZE: (u16, u16);

    fn init<DI, RST, DELAY>(
        display: &mut Display<Self, DI, RST>,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        DI: WriteOnlyDataCommand,
        RST: OutputPin,
        DELAY: DelayUs<u32>;

    fn write_pixels<DI, RST, I>(
        display: &mut Display<Self, DI, RST>,
        colors: I,
    ) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        RST: OutputPin,
        I: IntoIterator<Item = Self::Color>;
}

mod private {
    pub trait Sealed {}
}
