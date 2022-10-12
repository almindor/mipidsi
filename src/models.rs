use crate::{
    error::InitError, instruction::Instruction, ColorOrder, DisplayOptions, Error, Orientation,
};
use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

// existing model implementations
mod ili9342c;
mod ili9486;
mod st7735s;
mod st7789;

pub use ili9342c::*;
pub use ili9486::*;
pub use st7735s::*;
pub use st7789::*;

///
/// [DisplayOptions] that have been initialized with at minimum `display_size`
/// values. This protects against initializing a model with 0 size.
/// This structure also holds possible windowing offset values in case of
/// clipped displays such as the `Pico1`
///
#[derive(Debug, Clone)]
pub struct ModelOptions {
    /// Display options
    display_options: DisplayOptions,
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
    pub fn with_display_size(
        display_options: DisplayOptions,
        width: u16,
        height: u16,
    ) -> ModelOptions {
        ModelOptions {
            display_options,
            window_offset_handler: no_offset,
            display_size: (width, height),
            framebuffer_size: (0, 0),
        }
    }

    ///
    /// Constructs a [ModelOptions] from [DisplayOptions]
    /// with given display and framebuffer sizes
    ///
    pub fn with_sizes(
        display_options: DisplayOptions,
        display_size: (u16, u16),
        framebuffer_size: (u16, u16),
    ) -> ModelOptions {
        ModelOptions {
            display_options,
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
        display_options: DisplayOptions,
        display_size: (u16, u16),
        framebuffer_size: (u16, u16),
        window_offset_handler: fn(Orientation) -> (u16, u16),
    ) -> ModelOptions {
        ModelOptions {
            display_options,
            window_offset_handler,
            display_size,
            framebuffer_size,
        }
    }

    ///
    /// Returns MADCTL register value for given display options
    ///
    pub fn madctl(&self) -> u8 {
        let mut value = self.display_options.orientation.value_u8();
        if self.display_options.invert_vertical_refresh {
            value |= 0b0001_0000;
        }
        match self.display_options.color_order {
            ColorOrder::Rgb => {}
            ColorOrder::Bgr => value |= 0b0000_1000,
        }
        if self.display_options.invert_horizontal_refresh {
            value |= 0b0000_0100;
        }

        value
    }

    ///
    /// Returns display size based on current orientation and display options.
    /// Used by models.
    ///
    pub fn display_size(&self, orientation: Orientation) -> (u16, u16) {
        Self::orient_size(self.display_size, orientation)
    }

    ///
    /// Returns framebuffer size based on current orientation and display options.
    /// Used by models. Uses display_size if framebuffer_size is not set.
    ///
    pub fn framebuffer_size(&self, orientation: Orientation) -> (u16, u16) {
        let size = if self.framebuffer_size == (0, 0) {
            self.display_size
        } else {
            self.framebuffer_size
        };

        Self::orient_size(size, orientation)
    }

    ///
    /// Returns window offset based on current orientation and display options.
    /// Used by [Display::set_address_window]
    ///
    pub fn window_offset(&self, orientation: Orientation) -> (u16, u16) {
        (self.window_offset_handler)(orientation)
    }

    pub fn orientation(&self) -> Orientation {
        self.display_options.orientation
    }

    // Flip size according to orientation, in general
    fn orient_size(size: (u16, u16), orientation: Orientation) -> (u16, u16) {
        match orientation {
            Orientation::Portrait(_) | Orientation::PortraitInverted(_) => size,
            Orientation::Landscape(_) | Orientation::LandscapeInverted(_) => (size.1, size.0),
        }
    }
}

pub trait Model {
    type ColorFormat: RgbColor;

    /// Common model constructor
    fn new(options: ModelOptions) -> Self;

    /// Initializes the display for this model
    /// and returns the value of MADCTL set by init
    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        rst: &mut Option<RST>,
    ) -> Result<u8, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand;

    fn hard_reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
    {
        rst.set_low().map_err(InitError::Pin)?;
        delay.delay_us(10);
        rst.set_high().map_err(InitError::Pin)?;

        Ok(())
    }

    /// Writes pixels to the display IC via the given DisplayInterface
    /// Any pixel color format conversion is done here
    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>;

    fn options(&self) -> &ModelOptions;
}

// helper for models
pub fn write_command<DI>(di: &mut DI, command: Instruction, params: &[u8]) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
{
    di.send_commands(DataFormat::U8(&[command as u8]))?;

    if !params.is_empty() {
        di.send_data(DataFormat::U8(params))?;
        Ok(())
    } else {
        Ok(())
    }
}
