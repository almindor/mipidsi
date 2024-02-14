//! [ModelOptions] and other helper types.

use crate::models::Model;

mod orientation;
pub(crate) use orientation::MemoryMapping;
pub use orientation::{InvalidAngleError, Orientation, Rotation};

/// [ModelOptions] holds the settings for [Model]s.
///
/// `display_size` being set is the minimum requirement.
#[derive(Clone)]
pub struct ModelOptions {
    /// Specify display color ordering
    pub(crate) color_order: ColorOrder,
    /// Initial display orientation (without inverts)
    pub(crate) orientation: Orientation,
    /// Whether to invert colors for this display/model (INVON)
    pub(crate) invert_colors: ColorInversion,
    /// Display refresh order
    pub(crate) refresh_order: RefreshOrder,
    /// Display size (w, h) for given display/model
    pub(crate) display_size: (u16, u16),
    /// Display offset (x, y) for given display/model
    pub(crate) display_offset: (u16, u16),
}

impl ModelOptions {
    /// Creates model options for the entire framebuffer.
    pub fn full_size<M: Model>() -> Self {
        Self {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_colors: ColorInversion::default(),
            refresh_order: RefreshOrder::default(),
            display_size: M::FRAMEBUFFER_SIZE,
            display_offset: (0, 0),
        }
    }

    /// Creates model options for the given size and offset.
    pub fn with_all(display_size: (u16, u16), display_offset: (u16, u16)) -> Self {
        Self {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_colors: ColorInversion::default(),
            refresh_order: RefreshOrder::default(),
            display_size,
            display_offset,
        }
    }

    /// Sets the color inversion setting.
    pub fn set_invert_colors(&mut self, color_inversion: ColorInversion) {
        self.invert_colors = color_inversion;
    }

    /// Returns the display size based on current orientation and display options.
    ///
    /// Used by models.
    pub(crate) fn display_size(&self) -> (u16, u16) {
        if self.orientation.rotation.is_horizontal() {
            self.display_size
        } else {
            (self.display_size.1, self.display_size.0)
        }
    }

    /// Returns the current orientation.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Sets the orientation.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}

/// Color inversion.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorInversion {
    /// Normal colors.
    Normal,
    /// Inverted colors.
    Inverted,
}

impl Default for ColorInversion {
    fn default() -> Self {
        Self::Normal
    }
}

/// Vertical refresh order.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerticalRefreshOrder {
    /// Refresh from top to bottom.
    TopToBottom,
    /// Refresh from bottom to top.
    BottomToTop,
}

impl Default for VerticalRefreshOrder {
    fn default() -> Self {
        Self::TopToBottom
    }
}

impl VerticalRefreshOrder {
    /// Returns the opposite refresh order.
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::TopToBottom => Self::BottomToTop,
            Self::BottomToTop => Self::TopToBottom,
        }
    }
}

/// Horizontal refresh order.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HorizontalRefreshOrder {
    /// Refresh from left to right.
    LeftToRight,
    /// Refresh from right to left.
    RightToLeft,
}

impl Default for HorizontalRefreshOrder {
    fn default() -> Self {
        Self::LeftToRight
    }
}

impl HorizontalRefreshOrder {
    /// Returns the opposite refresh order.
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::LeftToRight => Self::RightToLeft,
            Self::RightToLeft => Self::LeftToRight,
        }
    }
}

/// Display refresh order.
///
/// Defaults to left to right, top to bottom.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct RefreshOrder {
    /// Vertical refresh order.
    pub vertical: VerticalRefreshOrder,
    /// Horizontal refresh order.
    pub horizontal: HorizontalRefreshOrder,
}

impl RefreshOrder {
    /// Creates a new refresh order.
    pub const fn new(vertical: VerticalRefreshOrder, horizontal: HorizontalRefreshOrder) -> Self {
        Self {
            vertical,
            horizontal,
        }
    }

    /// Returns a refresh order with flipped vertical refresh order.
    #[must_use]
    pub const fn flip_vertical(self) -> Self {
        Self {
            vertical: self.vertical.flip(),
            ..self
        }
    }

    /// Returns a refresh order with flipped horizontal refresh order.
    #[must_use]
    pub const fn flip_horizontal(self) -> Self {
        Self {
            horizontal: self.horizontal.flip(),
            ..self
        }
    }
}

/// Tearing effect output setting.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TearingEffect {
    /// Disable output.
    Off,
    /// Output vertical blanking information.
    Vertical,
    /// Output horizontal and vertical blanking information.
    HorizontalAndVertical,
}

/// Subpixel order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOrder {
    /// RGB subpixel order.
    Rgb,
    /// BGR subpixel order.
    Bgr,
}

impl Default for ColorOrder {
    fn default() -> Self {
        Self::Rgb
    }
}
