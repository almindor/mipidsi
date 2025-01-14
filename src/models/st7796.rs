use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;

use crate::{dcs::SetAddressMode, interface::Interface, models::Model, options::ModelOptions};

/// ST7796 display in Rgb565 color mode.
pub struct ST7796;

impl Model for ST7796 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 480);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, DI::Error>
    where
        DELAY: DelayNs,
        DI: Interface,
    {
        super::ST7789.init(di, delay, options)
    }
}
