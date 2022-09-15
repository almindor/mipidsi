use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital::v2::OutputPin;

use crate::{models::Model, Display, DisplayOptions, Orientation};

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
        Self::with_model(di, rst, ST7789::new(options.with_display_size(240, 320)))
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
        Self::with_model(di, rst, ST7789::new(options.with_display_size(240, 240)))
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
    pub fn st7789_pico1(di: DI, rst: Option<RST>, mut options: DisplayOptions) -> Self {
        // pico v1 is cropped to 135x240 size with an offset of (40, 53)
        options.window_offset_handler = pico1_offset;
        options.display_size = (135, 240);
        options.framebuffer_size = (135, 240);
        Self::with_model(di, rst, ST7789::new(options))
    }
}

// ST7789 pico1 variant with variable offset
pub(crate) fn pico1_offset(orientation: Orientation) -> (u16, u16) {
    // PortraitInverted(false) and Landscape(true) x offset is 53,
    // for Landscape(false) and PortraitInverted(true) it is 52
    match orientation {
        Orientation::Landscape(true) | Orientation::PortraitInverted(false) => (53, 40),
        _ => (52, 40),
    }
}
