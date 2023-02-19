//! [ModelOptions] and other helper types.

/// [ModelOptions] holds the settings for [Model](crate::Model)s.
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
    /// Offset override function returning (w, h) offset for current
    /// display orientation if display is "clipped" and needs an offset for (e.g. Pico v1)
    pub(crate) window_offset_handler: fn(&ModelOptions) -> (u16, u16),
    /// Display size (w, h) for given display/model
    pub(crate) display_size: (u16, u16),
    /// Framebuffer size (w, h) for given display/model
    pub(crate) framebuffer_size: (u16, u16),
}

impl ModelOptions {
    /// Creates model options for the given display and framebuffer sizes.
    ///
    /// All other settings are initialized to their default value.
    pub fn with_sizes(display_size: (u16, u16), framebuffer_size: (u16, u16)) -> Self {
        Self {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_colors: ColorInversion::default(),
            refresh_order: RefreshOrder::default(),
            window_offset_handler: no_offset,
            display_size,
            framebuffer_size,
        }
    }

    /// Creates model options for the given sizes and offset handler.
    pub fn with_all(
        display_size: (u16, u16),
        framebuffer_size: (u16, u16),
        window_offset_handler: fn(&ModelOptions) -> (u16, u16),
    ) -> Self {
        Self {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_colors: ColorInversion::default(),
            refresh_order: RefreshOrder::default(),
            window_offset_handler,
            display_size,
            framebuffer_size,
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
        Self::orient_size(self.display_size, self.orientation())
    }

    /// Returns framebuffer size based on current orientation and display options.
    ///
    /// Used by models. Uses display_size if framebuffer_size is not set.
    pub(crate) fn framebuffer_size(&self) -> (u16, u16) {
        let size = if self.framebuffer_size == (0, 0) {
            self.display_size
        } else {
            self.framebuffer_size
        };

        Self::orient_size(size, self.orientation())
    }

    /// Returns the larger of framebuffer width or height.
    ///
    /// Used for scroll area setups.
    pub(crate) fn framebuffer_size_max(&self) -> u16 {
        let (w, h) = self.framebuffer_size();

        w.max(h)
    }

    /// Returns window offset (x, y) based on current orientation and display options.
    ///
    /// Used by [Display::set_address_window](crate::Display::set_address_window).
    pub(crate) fn window_offset(&mut self) -> (u16, u16) {
        (self.window_offset_handler)(self)
    }

    /// Returns the current orientation.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Sets the orientation.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    // Flip size according to orientation, in general
    fn orient_size(size: (u16, u16), orientation: Orientation) -> (u16, u16) {
        match orientation {
            Orientation::Portrait(_) | Orientation::PortraitInverted(_) => size,
            Orientation::Landscape(_) | Orientation::LandscapeInverted(_) => (size.1, size.0),
        }
    }
}

///
/// `no_offset` is the default offset provider. It results to 0, 0 in case display_size is == framebuffer_size
/// and to framebuffer_size - display_size otherwise.
///
fn no_offset(options: &ModelOptions) -> (u16, u16) {
    // do FB size - Display size offset for inverted setups
    match options.orientation {
        Orientation::PortraitInverted(_) | Orientation::LandscapeInverted(_) => {
            let hdiff = options.framebuffer_size.1 - options.display_size.1;

            let mut x = 0;
            let mut y = 0;

            match options.orientation {
                Orientation::PortraitInverted(_) => y = hdiff,
                Orientation::LandscapeInverted(_) => x = hdiff,
                _ => {}
            }

            (x, y)
        }
        _ => (0, 0),
    }
}

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
