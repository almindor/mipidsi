//! [super::Display] builder module

use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital;
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
///     .with_reset_pin(rst)
///     .with_color_order(ColorOrder::Bgr)
///     .with_display_size(320, 240)
///     .init(&mut delay).unwrap();
/// ```
pub struct Builder<DI, MODEL, RST>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    di: DI,
    model: MODEL,
    rst: Option<RST>,
    options: ModelOptions,
}

impl<DI, MODEL> Builder<DI, MODEL, NoResetPin>
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
            rst: None,
            options: ModelOptions::full_size::<MODEL>(),
        }
    }
}

impl<DI, MODEL, RST> Builder<DI, MODEL, RST>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
    RST: OutputPin,
{
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

    /// Sets the reset pin.
    pub fn with_reset_pin<RST2: OutputPin>(self, rst: RST2) -> Builder<DI, MODEL, RST2> {
        Builder {
            di: self.di,
            model: self.model,
            rst: Some(rst),
            options: self.options,
        }
    }

    ///
    /// Consumes the builder to create a new [Display] with an optional reset [OutputPin].
    /// Blocks using the provided [DelayUs] `delay_source` to perform the display initialization.
    /// The display will be awake ready to use, no need to call [Display::wake] after init.
    ///
    /// ### WARNING
    /// The reset pin needs to be in *high* state in order for the display to operate.
    /// If it wasn't provided the user needs to ensure this is the case.
    pub fn init(
        mut self,
        delay_source: &mut impl DelayNs,
    ) -> Result<Display<DI, MODEL, RST>, InitError<RST::Error>> {
        let mut dcs = Dcs::write_only(self.di);
        let madctl = self
            .model
            .init(&mut dcs, delay_source, &self.options, &mut self.rst)?;

        let display = Display {
            dcs,
            model: self.model,
            rst: self.rst,
            options: self.options,
            madctl,
            sleeping: false, // TODO: init should lock state
        };

        Ok(display)
    }
}

/// Marker type for no reset pin.
pub enum NoResetPin {}

impl digital::OutputPin for NoResetPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl digital::ErrorType for NoResetPin {
    type Error = core::convert::Infallible;
}

#[cfg(test)]
mod tests {
    use crate::{
        _mock::{MockDelay, MockDisplayInterface, MockOutputPin},
        models::ILI9341Rgb565,
    };

    use super::*;

    #[test]
    fn init_without_reset_pin() {
        let _: Display<_, _, NoResetPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .init(&mut MockDelay)
            .unwrap();
    }

    #[test]
    fn init_with_reset_pin() {
        let _: Display<_, _, MockOutputPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .with_reset_pin(MockOutputPin)
            .init(&mut MockDelay)
            .unwrap();
    }
}
