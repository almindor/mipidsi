//! Module holding [ModelOptions] and other helper types for [super::Display]

///
/// [ModelOptions] hold all the various settings that can impact a particular [super::Model]
/// `display_size` being set is the minimum requirement.
///
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
    ///
    /// Constructs a [ModelOptions]
    /// with given display and framebuffer sizes
    ///
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

    ///
    /// Constructs a [ModelOptions]
    /// with given display and framebuffer sizes and provided window offset handler
    ///
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

    pub fn with_invert_colors(mut self, color_inversion: ColorInversion) -> Self {
        self.invert_colors = color_inversion;
        self
    }

    ///
    /// Returns display size based on current orientation and display options.
    /// Used by models.
    ///
    pub fn display_size(&self) -> (u16, u16) {
        Self::orient_size(self.display_size, self.orientation())
    }

    ///
    /// Returns framebuffer size based on current orientation and display options.
    /// Used by models. Uses display_size if framebuffer_size is not set.
    ///
    pub fn framebuffer_size(&self) -> (u16, u16) {
        let size = if self.framebuffer_size == (0, 0) {
            self.display_size
        } else {
            self.framebuffer_size
        };

        Self::orient_size(size, self.orientation())
    }

    ///
    /// Returns the larger of framebuffer width or height. Used for scroll
    /// area setups.
    ///
    pub fn framebuffer_size_max(&self) -> u16 {
        let (w, h) = self.framebuffer_size();

        w.max(h)
    }

    ///
    /// Returns window offset (x, y) based on current orientation and display options.
    /// Used by [Display::set_address_window]
    ///
    pub fn window_offset(&mut self) -> (u16, u16) {
        (self.window_offset_handler)(self)
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    ///
    /// Sets the current [Orientation]
    ///
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorInversion {
    Normal,
    Inverted,
}

impl Default for ColorInversion {
    fn default() -> Self {
        Self::Normal
    }
}

///
/// Display refresh order, defaults to left to right, top to bottom
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RefreshOrder {
    /// Left to Right and Top to Bottom
    Normal,
    /// Right to Left and Top to Bottom
    RightToLeft,
    /// Left to Right and Bottom to Top
    BottomToTop,
    /// Right to Left and Bottom to Top
    Inverted,
}

impl Default for RefreshOrder {
    fn default() -> Self {
        Self::Normal
    }
}

///
/// Tearing effect output setting.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
