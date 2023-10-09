//! [super::Display] builder module

use core::marker::PhantomData;

use display_interface::WriteOnlyDataCommand;
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    controllers::{Controller, NoResetPin},
    dcs::Dcs,
    error::InitError,
    ColorInversion, ColorOrder, Display, Orientation, RefreshOrder,
};

/// Builder for [Display] instances.
///
/// Exposes all possible display options.
///
///
/// ## Example
/// ```rust ignore
/// let mut display = Builder::ili9342c_rgb565(di)
///     .with_color_order(ColorOrder::Bgr);
///     .with_display_size(320, 240);
///     .init(&mut delay, Some(rst)).unwrap();
/// ```
pub struct Builder<C, DI, RST> {
    display: Display<C, DI, RST>,
}

impl<DI: WriteOnlyDataCommand> Builder<(), DI, NoResetPin> {
    ///
    /// Constructs a new builder from given [WriteOnlyDataCommand], [Model]
    /// and [ModelOptions]. For use by [Model] helpers, not public
    ///
    #[must_use]
    pub fn new<C: Controller>(di: DI) -> Builder<C, DI, NoResetPin> {
        Builder {
            display: Display {
                controller: PhantomData,
                dcs: Dcs::write_only(di),
                rst: None,
                size: C::FRAMEBUFFER_SIZE,
                offset: (0, 0),
                color_order: ColorOrder::default(),
                orientation: Orientation::default(),
                invert_colors: ColorInversion::default(),
                refresh_order: RefreshOrder::default(),
            },
        }
    }
}

impl<C, DI, RST> Builder<C, DI, RST>
where
    C: Controller,
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Sets the invert color flag
    ///
    #[must_use]
    pub fn invert_colors(mut self, invert_colors: ColorInversion) -> Self {
        self.display.invert_colors = invert_colors;
        self
    }

    ///
    /// Sets the [ColorOrder]
    ///
    #[must_use]
    pub fn color_order(mut self, color_order: ColorOrder) -> Self {
        self.display.color_order = color_order;
        self
    }

    ///
    /// Sets the [Orientation]
    ///
    #[must_use]
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.display.orientation = orientation;
        self
    }

    ///
    /// Sets refresh order
    ///
    #[must_use]
    pub fn refresh_order(mut self, refresh_order: RefreshOrder) -> Self {
        self.display.refresh_order = refresh_order;
        self
    }

    /// Sets the reset pin.
    #[must_use]
    pub fn reset_pin<RST2: OutputPin>(self, reset_pin: RST2) -> Builder<C, DI, RST2> {
        Builder {
            display: Display {
                dcs: self.display.dcs,
                invert_colors: self.display.invert_colors,
                size: self.display.size,
                offset: self.display.offset,
                color_order: self.display.color_order,
                orientation: self.display.orientation,
                refresh_order: self.display.refresh_order,
                controller: PhantomData,
                rst: Some(reset_pin),
            },
        }
    }

    ///
    /// Sets the display size
    ///
    #[must_use]
    pub fn display_size(mut self, width: u16, height: u16) -> Self {
        self.display.size = (width, height);
        self
    }

    ///
    /// Sets the display offset
    ///
    #[must_use]
    pub fn display_offset(mut self, x: u16, y: u16) -> Self {
        self.display.offset = (x, y);
        self
    }

    ///
    /// Consumes the builder to create a new [Display] with an optional reset [OutputPin].
    /// Blocks using the provided [DelayUs] `delay_source` to perform the display initialization.
    /// The display will be awake ready to use, no need to call [Display::wake] after init.
    ///
    /// ### WARNING
    /// The reset pin needs to be in *high* state in order for the display to operate.
    /// If it wasn't provided the user needs to ensure this is the case.
    #[must_use]
    pub fn init(
        mut self,
        delay_source: &mut impl DelayUs<u32>,
    ) -> Result<Display<C, DI, RST>, InitError<RST::Error>> {
        C::init(&mut self.display, delay_source)?;

        Ok(self.display)
    }
}
