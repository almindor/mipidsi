use display_interface::WriteOnlyDataCommand;

use crate::{Builder, ModelOptions};

use super::ST7789;

impl<DI> Builder<DI, ST7789>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ST7789] as the [super::Model] with
    /// general variant using display framebuffer size of 240x320
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn st7789(di: DI) -> Self {
        Self::with_model(di, ST7789)
    }

    ///
    /// Creates a new [Display] instance with [ST7789] as the [super::Model] with
    /// pico1 variant using display and framebuffer size of 135x240 and a clipping offset
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn st7789_pico1(di: DI) -> Self {
        // pico v1 is cropped to 135x240 size with an offset of (40, 53)
        Self::new(
            di,
            ST7789,
            ModelOptions::with_all((135, 240), (135, 240), pico1_offset),
        )
    }
}

// ST7789 pico1 variant with variable offset
pub(crate) fn pico1_offset(options: &ModelOptions) -> (u16, u16) {
    (0, 0)

    //TODO: fix
    // match options.orientation() {
    //     Orientation::Portrait(false) => (52, 40),
    //     Orientation::Portrait(true) => (53, 40),
    //     Orientation::Landscape(false) => (40, 52),
    //     Orientation::Landscape(true) => (40, 53),
    //     Orientation::PortraitInverted(false) => (53, 40),
    //     Orientation::PortraitInverted(true) => (52, 40),
    //     Orientation::LandscapeInverted(false) => (40, 53),
    //     Orientation::LandscapeInverted(true) => (40, 52),
    // }
}
