//! [ModelOptions] and other helper types.

///
/// Display orientation.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
