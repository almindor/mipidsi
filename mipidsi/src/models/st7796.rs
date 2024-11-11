use display_interface::WriteOnlyDataCommand;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::{delay::DelayNs, digital::OutputPin};

use crate::{
    dcs::{Dcs, SetAddressMode},
    error::{Error, InitError},
    models::Model,
    options::ModelOptions,
};

/// ST7796 display in Rgb565 color mode.
///
/// Interfaces implemented by the [display-interface](https://crates.io/crates/display-interface) are supported.
pub struct ST7796;

impl Model for ST7796 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 480);

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
        super::ST7789.init(dcs, delay, options, rst)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        super::ST7789.write_pixels(dcs, colors)
    }

    fn repeat_pixel_to_buffer(color: Self::ColorFormat, buf: &mut [u8]) -> Result<usize, Error> {
        crate::graphics::repeat_pixel_to_buffer_rgb565(color, buf)
    }
}
