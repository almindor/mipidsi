//! Display models.

use crate::{
    dcs::{Dcs, SetAddressMode},
    error::Error,
    options::ModelOptions,
};
use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::delay::DelayNs;

// existing model implementations
mod gc9a01;
mod ili9341;
mod ili9342c;
mod ili934x;
mod ili9486;
mod st7735s;
mod st7789;
mod st7796;

pub use gc9a01::*;
pub use ili9341::*;
pub use ili9342c::*;
pub use ili9486::*;
pub use st7735s::*;
pub use st7789::*;
pub use st7796::*;

///
/// Endianness indicator enum for use with [Model] as an associated constant.
/// This allows us to know what format of data the display expects.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    /// Big endian
    BigEndian,
    /// Little endian
    LittleEndian,
}

/// Display model.
pub trait Model {
    /// The color format.
    type ColorFormat: RgbColor;

    /// The framebuffer size in pixels.
    const FRAMEBUFFER_SIZE: (u16, u16);

    /// Endianness expectation of the display Model's data, defaults to big endian
    const ENDIANNESS: Endianness = Endianness::BigEndian;

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    fn init<DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, Error>
    where
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand;

    /// Writes pixels to the display IC via the given display interface.
    ///
    /// Any pixel color format conversion is done here.
    fn write_pixels<DI, I>(&mut self, di: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>;
}
