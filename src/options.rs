//! Module holding [ModelOptions] and other helper types for [super::Display]

///
/// [DisplayOptions] that have been initialized with at minimum `display_size`
/// values. This protects against initializing a model with 0 size.
/// This structure also holds possible windowing offset values in case of
/// clipped displays such as the `Pico1`
///
#[derive(Debug, Clone)]
pub(crate) struct ModelOptions {
    /// Specify display color ordering
    color_order: ColorOrder,
    /// Initial display orientation (without inverts)
    pub(crate) orientation: Orientation,
    /// Set to make display vertical refresh bottom to top
    invert_vertical_refresh: bool,
    /// Set to make display horizontal refresh right to left
    invert_horizontal_refresh: bool,
    /// Offset override function returning (w, h) offset for current
    /// display orientation if display is "clipped" and needs an offset for (e.g. Pico v1)
    window_offset_handler: fn(Orientation) -> (u16, u16),
    /// Display size (w, h) override for the display/model, (0, 0) for no override
    display_size: (u16, u16),
    /// Framebuffer size (w, h) override for the display/model, (0, 0) for no override
    framebuffer_size: (u16, u16),
}

fn no_offset(_: Orientation) -> (u16, u16) {
    (0, 0)
}

impl ModelOptions {
    ///
    /// Constructs a [ModelOptions] from [DisplayOptions]
    /// with given display size
    ///
    pub fn with_display_size(width: u16, height: u16) -> ModelOptions {
        ModelOptions {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_horizontal_refresh: false,
            invert_vertical_refresh: false,
            window_offset_handler: no_offset,
            display_size: (width, height),
            framebuffer_size: (0, 0),
        }
    }

    ///
    /// Constructs a [ModelOptions] from [DisplayOptions]
    /// with given display and framebuffer sizes
    ///
    pub fn with_sizes(display_size: (u16, u16), framebuffer_size: (u16, u16)) -> ModelOptions {
        ModelOptions {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_horizontal_refresh: false,
            invert_vertical_refresh: false,
            window_offset_handler: no_offset,
            display_size,
            framebuffer_size,
        }
    }

    ///
    /// Constructs a [ModelOptions] from [DisplayOptions]
    /// with given display and framebuffer sizes and provided window offset handler
    ///
    pub fn with_all(
        display_size: (u16, u16),
        framebuffer_size: (u16, u16),
        window_offset_handler: fn(Orientation) -> (u16, u16),
    ) -> ModelOptions {
        ModelOptions {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_horizontal_refresh: false,
            invert_vertical_refresh: false,
            window_offset_handler,
            display_size,
            framebuffer_size,
        }
    }

    ///
    /// Returns MADCTL register value for given display options
    ///
    pub fn madctl(&self) -> u8 {
        let mut value = self.orientation.value_u8();
        if self.invert_vertical_refresh {
            value |= 0b0001_0000;
        }
        match self.color_order {
            ColorOrder::Rgb => {}
            ColorOrder::Bgr => value |= 0b0000_1000,
        }
        if self.invert_horizontal_refresh {
            value |= 0b0000_0100;
        }

        value
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
    /// Returns window offset based on current orientation and display options.
    /// Used by [Display::set_address_window]
    ///
    pub fn window_offset(&self) -> (u16, u16) {
        (self.window_offset_handler)(self.orientation())
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
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
