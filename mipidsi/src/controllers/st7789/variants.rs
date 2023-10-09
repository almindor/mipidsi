use display_interface::WriteOnlyDataCommand;

use crate::{Builder, ColorInversion, ModelOptions, Orientation};

use super::ST7789;

impl<DI> Builder<DI, ST7789>
where
    DI: WriteOnlyDataCommand,
{
    /// Creates a new display builder for a ST7789 display in Rgb565 color mode.
    ///
    /// The default framebuffer size and display size is 240x320 pixels.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](WriteOnlyDataCommand) for communicating with the display
    ///
    pub fn st7789(di: DI) -> Self {
        Self::with_model(di, ST7789)
    }

    /// Creates a new display builder for the pico1 variant of a ST7789 display in Rgb565 color
    /// mode.
    ///
    /// The pico1 variant uses a display and framebuffer size of 135x240 and a clipping offset.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](WriteOnlyDataCommand) for communicating with the display
    ///
    pub fn st7789_pico1(di: DI) -> Self {
        let mut options = ModelOptions::with_all((135, 240), (135, 240), pico1_offset);
        options.set_invert_colors(ColorInversion::Inverted);

        // pico v1 is cropped to 135x240 size with an offset of (40, 53)
        Self::new(di, ST7789, options)
    }
}

// ST7789 pico1 variant with variable offset
pub(crate) fn pico1_offset(options: &ModelOptions) -> (u16, u16) {
    match options.orientation() {
        Orientation::Portrait(false) => (52, 40),
        Orientation::Portrait(true) => (53, 40),
        Orientation::Landscape(false) => (40, 52),
        Orientation::Landscape(true) => (40, 53),
        Orientation::PortraitInverted(false) => (53, 40),
        Orientation::PortraitInverted(true) => (52, 40),
        Orientation::LandscapeInverted(false) => (40, 53),
        Orientation::LandscapeInverted(true) => (40, 52),
    }
}
