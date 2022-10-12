//! Module holding [DisplayOptions] and other helper types for [super::Display]

///
/// Options for displays used on initialization
///
#[derive(Debug, Clone, Default)]
pub struct DisplayOptions {
    /// Initial display orientation (without inverts)
    pub orientation: Orientation,
    /// Set to make display vertical refresh bottom to top
    pub invert_vertical_refresh: bool,
    /// Specify display color ordering
    pub color_order: ColorOrder,
    /// Set to make display horizontal refresh right to left
    pub invert_horizontal_refresh: bool,
}

///
/// Display orientation.
///
#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    /// Portrait orientation, with mirror image parameter
    Portrait(bool),
    /// Landscape orientation, with mirror image parameter
    Landscape(bool),
    /// Inverted Portrait orientation, with mirror image parameter
    PortraitInverted(bool),
    /// Inverted Lanscape orientation, with mirror image parameter
    LandscapeInverted(bool),
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait(false)
    }
}

impl Orientation {
    pub fn value_u8(&self) -> u8 {
        match self {
            Orientation::Portrait(false) => 0b0000_0000,
            Orientation::Portrait(true) => 0b0100_0000,
            Orientation::PortraitInverted(false) => 0b1100_0000,
            Orientation::PortraitInverted(true) => 0b1000_0000,
            Orientation::Landscape(false) => 0b0010_0000,
            Orientation::Landscape(true) => 0b0110_0000,
            Orientation::LandscapeInverted(false) => 0b1110_0000,
            Orientation::LandscapeInverted(true) => 0b1010_0000,
        }
    }
}

///
/// Tearing effect output setting.
///
#[derive(Copy, Clone)]
pub enum TearingEffect {
    /// Disable output.
    Off,
    /// Output vertical blanking information.
    Vertical,
    /// Output horizontal and vertical blanking information.
    HorizontalAndVertical,
}

///
/// Defines expected color component ordering, RGB or BGR
///
#[derive(Debug, Clone, Copy)]
pub enum ColorOrder {
    Rgb,
    Bgr,
}

impl Default for ColorOrder {
    fn default() -> Self {
        Self::Rgb
    }
}
