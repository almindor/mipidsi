//! [super::Display] builder module

use display_interface::WriteOnlyDataCommand;
use embedded_hal::{delay::DelayNs, digital::OutputPin};

use crate::{dcs::Dcs, error::InitError, models::Model, Display};

use crate::options::{ColorInversion, ColorOrder, ModelOptions, Orientation, RefreshOrder};

/// Builder for [Display] instances.
///
/// Exposes all possible display options.
///
/// # Examples
///
/// ```
/// use mipidsi::{Builder, options::ColorOrder, models::ILI9342CRgb565};
///
/// # let di = mipidsi::_mock::MockDisplayInterface;
/// # let rst = mipidsi::_mock::MockOutputPin;
/// # let mut delay = mipidsi::_mock::MockDelay;
/// let mut display = Builder::new(ILI9342CRgb565, di)
///     .with_color_order(ColorOrder::Bgr)
///     .with_display_size(320, 240)
///     .init(&mut delay, Some(rst)).unwrap();
/// ```
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
    /// Constructs a new builder for given [Model].
    ///
    pub fn new(model: MODEL, di: DI) -> Self {
        Self {
            di,
            model,
            options: ModelOptions::full_size::<MODEL>(),
        }
    }

    ///
    /// Sets the invert color flag
    ///
    pub fn with_invert_colors(mut self, color_inversion: ColorInversion) -> Self {
        self.options.invert_colors = color_inversion;
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
    /// Sets refresh order
    ///
    pub fn with_refresh_order(mut self, refresh_order: RefreshOrder) -> Self {
        self.options.refresh_order = refresh_order;
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
    /// Sets the display offset
    ///
    pub fn with_display_offset(mut self, x: u16, y: u16) -> Self {
        self.options.display_offset = (x, y);
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
    pub fn init<RST>(
        mut self,
        delay_source: &mut impl DelayNs,
        mut rst: Option<RST>,
    ) -> Result<Display<DI, MODEL, RST>, InitError<RST::Error>>
    where
        RST: OutputPin,
    {
        let mut dcs = Dcs::write_only(self.di);
        let madctl = self
            .model
            .init(&mut dcs, delay_source, &self.options, &mut rst)?;
        let display = Display {
            dcs,
            model: self.model,
            rst,
            options: self.options,
            madctl,
            sleeping: false, // TODO: init should lock state
        };

        Ok(display)
    }
}
