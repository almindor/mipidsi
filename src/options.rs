//! Module holding [ModelOptions] and other helper types for [super::Display]

use display_driver_hal::{Orientation, Rotation};

use crate::orientation_to_madctl;

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
    /// Set to make display vertical refresh bottom to top
    pub(crate) invert_vertical_refresh: bool,
    /// Set to make display horizontal refresh right to left
    pub(crate) invert_horizontal_refresh: bool,
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
            invert_horizontal_refresh: false,
            invert_vertical_refresh: false,
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
        let mut value = orientation_to_madctl(self.orientation);

        if self.invert_vertical_refresh {
            value |= 1 << 4
        }

        match self.color_order {
            ColorOrder::Rgb => {}
            ColorOrder::Bgr => value |= 1 << 3,
        }

        if self.invert_horizontal_refresh {
            value |= 1 << 2;
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
        match orientation.rotation {
            Rotation::Deg0 | Rotation::Deg180 => size,
            Rotation::Deg90 | Rotation::Deg270 => (size.1, size.0),
        }
    }
}

///
/// `no_offset` is the default offset provider. It results to 0, 0 in case display_size is == framebuffer_size
/// and to framebuffer_size - display_size otherwise.
///
fn no_offset(options: &ModelOptions) -> (u16, u16) {
    //TODO: fix
    (0, 0)
    // // do FB size - Display size offset for inverted setups
    // match options.orientation {
    //     Orientation::PortraitInverted(_) | Orientation::LandscapeInverted(_) => {
    //         let hdiff = options.framebuffer_size.1 - options.display_size.1;

    //         let mut x = 0;
    //         let mut y = 0;

    //         match options.orientation {
    //             Orientation::PortraitInverted(_) => y = hdiff,
    //             Orientation::LandscapeInverted(_) => x = hdiff,
    //             _ => {}
    //         }

    //         (x, y)
    //     }
    //     _ => (0, 0),
    // }
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
