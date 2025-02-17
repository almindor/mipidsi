use embedded_graphics_core::pixelcolor::Rgb565;

use crate::{
    dcs::{
        BitsPerPixel, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode, SetDisplayOn,
        SetInvertMode, SetPixelFormat,
    },
    models::{Model, ModelInitError},
    options::ModelOptions,
};

use super::InitEngine;

/// ST7789 display in Rgb565 color mode.
pub struct ST7789;

impl Model for ST7789 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<IE>(
        &mut self,
        options: &ModelOptions,
        ie: &mut IE,
    ) -> Result<SetAddressMode, ModelInitError<IE::Error>>
    where
        IE: InitEngine,
    {
        // if !matches!(
        //     DI::KIND,
        //     InterfaceKind::Serial4Line | InterfaceKind::Parallel8Bit | InterfaceKind::Parallel16Bit
        // ) {
        //     return Err(ModelInitError::InvalidConfiguration(
        //         ConfigurationError::UnsupportedInterface,
        //     ));
        // }

        let madctl = SetAddressMode::from(options);

        ie.queue_delay_us(150_000)?;

        ie.queue_command(ExitSleepMode)?;
        ie.queue_delay_us(10_000)?;

        // set hw scroll area based on framebuffer size
        ie.queue_command(madctl)?;

        ie.queue_command(SetInvertMode::new(options.invert_colors))?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        ie.queue_command(SetPixelFormat::new(pf))?;
        ie.queue_delay_us(10_000)?;
        ie.queue_command(EnterNormalMode)?;
        ie.queue_delay_us(10_000)?;
        ie.queue_command(SetDisplayOn)?;

        // DISPON requires some time otherwise we risk SPI data issues
        ie.queue_delay_us(120_000)?;

        Ok(madctl)
    }
}
