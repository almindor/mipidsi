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
///     .reset_pin(rst)
///     .color_order(ColorOrder::Bgr)
///     .display_size(320, 240)
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
    #[must_use]
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
    #[must_use]
    pub fn invert_colors(mut self, color_inversion: ColorInversion) -> Self {
        self.options.invert_colors = color_inversion;
        self
    }

    ///
    /// Sets the [ColorOrder]
    ///
    #[must_use]
    pub fn color_order(mut self, color_order: ColorOrder) -> Self {
        self.options.color_order = color_order;
        self
    }

    ///
    /// Sets the [Orientation]
    ///
    #[must_use]
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.options.orientation = orientation;
        self
    }

    ///
    /// Sets refresh order
    ///
    #[must_use]
    pub fn refresh_order(mut self, refresh_order: RefreshOrder) -> Self {
        self.options.refresh_order = refresh_order;
        self
    }

    /// Sets the display size.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is 0.
    #[must_use]
    pub fn display_size(mut self, width: u16, height: u16) -> Self {
        assert!(width != 0 && height != 0);

        self.options.display_size = (width, height);
        self
    }

    ///
    /// Sets the display offset
    ///
    #[must_use]
    pub fn display_offset(mut self, x: u16, y: u16) -> Self {
        self.options.display_offset = (x, y);
        self
    }

    /// Sets the reset pin.
    ///
    /// ### WARNING
    /// The reset pin needs to be in *high* state in order for the display to operate.
    /// If it wasn't provided the user needs to ensure this is the case.
    ///
    #[must_use]
    pub fn reset_pin<RST2: OutputPin>(self, rst: RST2) -> Builder<DI, MODEL, RST2> {
        Builder {
            di: self.di,
            model: self.model,
            rst: Some(rst),
            options: self.options,
        }
    }

    ///
    /// Consumes the builder to create a new [Display] with an optional reset [OutputPin].
    /// Blocks using the provided [DelayNs] `delay_source` to perform the display initialization.
    /// The display will be awake ready to use, no need to call [Display::wake] after init.
    ///
    /// # Panics
    ///
    /// Panics if the area defined by the [`display_size`](Self::display_size)
    /// and [`display_offset`](Self::display_offset) settings is (partially)
    /// outside the framebuffer.
    pub fn init(
        mut self,
        delay_source: &mut impl DelayNs,
    ) -> Result<Display<DI, MODEL, RST>, InitError<RST::Error>> {
        let to_u32 = |(a, b)| (u32::from(a), u32::from(b));
        let (width, height) = to_u32(self.options.display_size);
        let (offset_x, offset_y) = to_u32(self.options.display_offset);
        let (max_width, max_height) = to_u32(MODEL::FRAMEBUFFER_SIZE);
        assert!(width + offset_x <= max_width);
        assert!(height + offset_y <= max_height);

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
    fn init_reset_pin() {
        let _: Display<_, _, MockOutputPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .reset_pin(MockOutputPin)
            .init(&mut MockDelay)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "assertion failed: width + offset_x <= max_width")]
    fn panic_too_wide() {
        let _: Display<_, _, MockOutputPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .reset_pin(MockOutputPin)
            .display_size(241, 320)
            .init(&mut MockDelay)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "assertion failed: height + offset_y <= max_height")]
    fn panic_too_high() {
        let _: Display<_, _, MockOutputPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .reset_pin(MockOutputPin)
            .display_size(240, 321)
            .init(&mut MockDelay)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "assertion failed: width + offset_x <= max_width")]
    fn panic_offset_invalid_x() {
        let _: Display<_, _, MockOutputPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .reset_pin(MockOutputPin)
            .display_size(240, 320)
            .display_offset(1, 0)
            .init(&mut MockDelay)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "assertion failed: height + offset_y <= max_height")]
    fn panic_offset_invalid_y() {
        let _: Display<_, _, MockOutputPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .reset_pin(MockOutputPin)
            .display_size(240, 310)
            .display_offset(0, 11)
            .init(&mut MockDelay)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "assertion failed: width != 0 && height != 0")]
    fn panic_zero_size() {
        let _: Display<_, _, MockOutputPin> = Builder::new(ILI9341Rgb565, MockDisplayInterface)
            .reset_pin(MockOutputPin)
            .display_size(0, 0)
            .init(&mut MockDelay)
            .unwrap();
    }
}
