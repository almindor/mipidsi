use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital::v2::OutputPin;

use crate::{
    models::{Model, ModelOptions},
    Display, DisplayOptions, Orientation,
};

use super::ST7789;

impl<DI, RST> Display<DI, RST, ST7789>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Creates a new [Display] instance with [ST7789] as the [Model] with
    /// general variant using display size of 240x320
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7789(di: DI, rst: Option<RST>, options: DisplayOptions) -> Self {
        Self::with_model(
            di,
            rst,
            ST7789::new(ModelOptions::with_display_size(options, 240, 320)),
        )
    }

    ///
    /// Creates a new [Display] instance with [ST7789] as the [Model] with
    /// general variant using display size of 240x240
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7789_240x240(di: DI, rst: Option<RST>, options: DisplayOptions) -> Self {
        Self::with_model(
            di,
            rst,
            ST7789::new(ModelOptions::with_display_size(options, 240, 240)),
        )
    }
    ///
    /// Creates a new [Display] instance with [ST7789] as the [Model] with
    /// general variant using display size of 240x240 but a frame buffer of 240x320 and adjusting the offset
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7789_240x240_b240x320(di: DI, rst: Option<RST>, options: DisplayOptions) -> Self {
        Self::with_model(
            di,
            rst,
            ST7789::new(ModelOptions::with_all(
                options,
                (240, 240),
                (240, 320),
                st7789_offset,
            )),
        )
    }

    ///
    /// Creates a new [Display] instance with [ST7789] as the [Model] with
    /// pico1 variant using display size of 135x240 and a clipping offset
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7789_pico1(di: DI, rst: Option<RST>, options: DisplayOptions) -> Self {
        // pico v1 is cropped to 135x240 size with an offset of (40, 53)
        Self::with_model(
            di,
            rst,
            ST7789::new(ModelOptions::with_all(
                options,
                (135, 240),
                (135, 240),
                pico1_offset,
            )),
        )
    }
}

// ST7789 pico1 variant with variable offset
pub(crate) fn pico1_offset(orientation: Orientation) -> (u16, u16) {
    match orientation {
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

// ST7789 pico1 variant with variable offset
pub(crate) fn st7789_offset(orientation: Orientation) -> (u16, u16) {
    match orientation {
        Orientation::Portrait(false) => (0, 0),
        Orientation::Portrait(true) => (0, 0),
        Orientation::Landscape(false) => (0, 0),
        Orientation::Landscape(true) => (0, 0),
        Orientation::PortraitInverted(false) => (0, 80),
        Orientation::PortraitInverted(true) => (0, 80),
        Orientation::LandscapeInverted(false) => (80, 0),
        Orientation::LandscapeInverted(true) => (80, 0),
    }
}
