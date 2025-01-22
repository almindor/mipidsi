use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::delay::DelayNs;

use crate::{
    dcs::{BitsPerPixel, PixelFormat, SetAddressMode},
    interface::Interface,
    options::ModelOptions,
};

use super::{ili948x, Model};

/// ILI9486 display in Rgb565 color mode.
pub struct ILI9486Rgb565;

/// ILI9486 display in Rgb666 color mode.
pub struct ILI9486Rgb666;

impl Model for ILI9486Rgb565 {
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
        assert_interface_kind!(Parallel8Bit | Parallel16Bit);

        delay.delay_us(120_000);

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili948x::init_common(di, delay, options, pf)
    }
}

impl Model for ILI9486Rgb666 {
    type ColorFormat = Rgb666;
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
        assert_interface_kind!(Serial4Line | Parallel8Bit | Parallel16Bit);

        delay.delay_us(120_000);

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ili948x::init_common(di, delay, options, pf)
    }
}
