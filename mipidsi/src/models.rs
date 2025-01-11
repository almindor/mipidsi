//! Display models.

use crate::{dcs::SetAddressMode, interface::Interface, options::ModelOptions};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::delay::DelayNs;

// existing model implementations
mod gc9107;
mod gc9a01;
mod ili9341;
mod ili9342c;
mod ili934x;
mod ili9486;
mod rm67162;
mod st7735s;
mod st7789;
mod st7796;

pub use gc9107::*;
pub use gc9a01::*;
pub use ili9341::*;
pub use ili9342c::*;
pub use ili9486::*;
pub use rm67162::*;
pub use st7735s::*;
pub use st7789::*;
pub use st7796::*;

/// Display model.
pub trait Model {
    /// The color format.
    type ColorFormat: RgbColor;

    /// The framebuffer size in pixels.
    const FRAMEBUFFER_SIZE: (u16, u16);

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, DI::Error>
    where
        DELAY: DelayNs,
        DI: Interface;
}
