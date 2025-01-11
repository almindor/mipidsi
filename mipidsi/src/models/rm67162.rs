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

/// RM67162 AMOLED display driver implementation
/// Supports:
/// - 16-bit RGB565 color
/// - 240x536 resolution
pub struct RM67162;

impl Model for RM67162 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 536);

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

        di.write_raw(0xFE, &[0x04])?; // SET APGE3
        di.write_raw(0x6A, &[0x00])?;
        di.write_raw(0xFE, &[0x05])?; // SET APGE4
        di.write_raw(0xFE, &[0x07])?; // SET APGE6
        di.write_raw(0x07, &[0x4F])?;
        di.write_raw(0xFE, &[0x01])?; // SET APGE0
        di.write_raw(0x2A, &[0x02])?; // Set column start address
        di.write_raw(0x2B, &[0x73])?; // Set row start address
        di.write_raw(0xFE, &[0x0A])?; // SET APGE9
        di.write_command(SetDisplayOn)?;
        di.write_raw(0xFE, &[0x00])?; // CMD Mode Switch to User Command Set
        di.write_raw(0x51, &[0xaf])?; // Set brightness
        di.write_raw(0x53, &[0x20])?; // Write CTRL display
        di.write_raw(0x35, &[0x00])?; // set Tearing Effect Line on

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        di.write_command(SetPixelFormat::new(pf))?;

        di.write_raw(0xC4, &[0x80])?; // set_DSPI Mode to SPI_WRAM

        di.write_command(madctl)?;

        di.write_command(SetInvertMode::new(options.invert_colors))?;

        di.write_command(ExitSleepMode)?; // turn off sleep
        delay.delay_us(120_000);

        di.write_command(SetDisplayOn)?; // turn on display

        Ok(madctl)
    }
}
