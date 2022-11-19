//! [super::Display] builder module

use display_interface::WriteOnlyDataCommand;
use embedded_hal::{delay::DelayUs, digital::OutputPin};

use crate::{error::InitError, models::Model, ColorOrder, Display, ModelOptions, Orientation};

///
/// Constructor helper for creating [Display] instances
/// Exposes all possible display options
///
pub struct Builder<DI, MODEL>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    di: DI,
    model: MODEL,
    options: ModelOptions,
}

impl<DI, MODEL> Builder<DI, MODEL>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    ///
    /// Constructs a new builder from given [WriteOnlyDataCommand], [Model]
    /// and [ModelOptions]. For use by [Model] helpers, not public
    ///
    pub(crate) fn new(di: DI, model: MODEL, options: ModelOptions) -> Self {
        Self { di, model, options }
    }

    ///
    /// Constructs a new builder for given [Model] using the model's
    /// `default_options`
    ///
    pub fn with_model(di: DI, model: MODEL) -> Self {
        Self {
            di,
            model,
            options: MODEL::default_options(),
        }
    }

    ///
    /// Sets the invert color flag
    ///
    pub fn with_invert_colors(mut self, invert_colors: bool) -> Self {
        self.options.invert_colors = invert_colors;
        self
    }

    ///
    /// Sets the [ColorOrder]
    ///
    pub fn with_color_order(mut self, color_order: ColorOrder) -> Self {
        self.options.color_order = color_order;
        self
    }

    ///
    /// Sets the [Orientation]
    ///
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.options.orientation = orientation;
        self
    }

    ///
    /// Inverts vertical refresh
    ///
    pub fn with_invert_vertical_refresh(mut self, invert: bool) -> Self {
        self.options.invert_vertical_refresh = invert;
        self
    }

    ///
    /// Inverts horizontal refresh
    ///
    pub fn with_invert_horizontal_refresh(mut self, invert: bool) -> Self {
        self.options.invert_horizontal_refresh = invert;
        self
    }

    ///
    /// Sets the display size
    ///
    pub fn with_display_size(mut self, width: u16, height: u16) -> Self {
        self.options.display_size = (width, height);
        self
    }

    ///
    /// Sets the framebuffer size
    ///
    pub fn with_framebuffer_size(mut self, width: u16, height: u16) -> Self {
        self.options.framebuffer_size = (width, height);
        self
    }

    ///
    /// Consumes the builder to create a new [Display] with an optional reset [OutputPin].
    /// Blocks using the provided [DelayUs] `delay_source` to perform the display initialization.
    /// ### WARNING
    /// The reset pin needs to be in *high* state in order for the display to operate.
    /// If it wasn't provided the user needs to ensure this is the case.
    ///
    pub fn init<RST, DELAY: DelayUs>(
        self,
        delay_source: &mut DELAY,
        rst: Option<RST>,
    ) -> Result<Display<DI, MODEL, RST>, InitError<RST::Error, DELAY::Error>>
    where
        RST: OutputPin,
    {
        let options = self.options;
        let madctl = options.madctl();

        let mut display = Display {
            di: self.di,
            model: self.model,
            rst,
            options,
            madctl,
        };

        display.init(delay_source)?;

        Ok(display)
    }
}
