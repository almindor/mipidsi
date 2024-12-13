use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;

use crate::{
    dcs::{
        BitsPerPixel, ExitSleepMode, InterfaceExt, PixelFormat, SetAddressMode, SetDisplayOn,
        SetInvertMode, SetPixelFormat,
    },
    interface::Interface,
    options::ModelOptions,
};

use super::Model;

/// GC9107 display in Rgb565 color mode.
pub struct GC9107;

impl Model for GC9107 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (128, 160);

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
        let madctl = SetAddressMode::from(options);

        delay.delay_us(200_000);

        di.write_command(madctl)?;

        di.write_raw(0xB0, &[0xC0])?;
        di.write_raw(0xB2, &[0x2F])?;
        di.write_raw(0xB3, &[0x03])?;
        di.write_raw(0xB6, &[0x19])?;
        di.write_raw(0xB7, &[0x01])?;

        di.write_raw(0xAC, &[0xCB])?;
        di.write_raw(0xAB, &[0x0E])?;

        di.write_raw(0xB4, &[0x04])?;

        di.write_raw(0xA8, &[0x19])?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        di.write_command(SetPixelFormat::new(pf))?;

        di.write_raw(0xB8, &[0x08])?;

        di.write_raw(0xE8, &[0x24])?;

        di.write_raw(0xE9, &[0x48])?;

        di.write_raw(0xEA, &[0x22])?;

        di.write_raw(0xC6, &[0x30])?;
        di.write_raw(0xC7, &[0x18])?;

        di.write_raw(
            0xF0,
            &[
                0x1F, 0x28, 0x04, 0x3E, 0x2A, 0x2E, 0x20, 0x00, 0x0C, 0x06, 0x00, 0x1C, 0x1F, 0x0f,
            ],
        )?;

        di.write_raw(
            0xF1,
            &[
                0x00, 0x2D, 0x2F, 0x3C, 0x6F, 0x1C, 0x0B, 0x00, 0x00, 0x00, 0x07, 0x0D, 0x11, 0x0f,
            ],
        )?;

        di.write_command(SetInvertMode::new(options.invert_colors))?;

        di.write_command(ExitSleepMode)?; // turn off sleep
        delay.delay_us(120_000);

        di.write_command(SetDisplayOn)?; // turn on display

        Ok(madctl)
    }
}
