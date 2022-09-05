#![no_std]
// associated re-typing not supported in rust yet
#![allow(clippy::type_complexity)]

//! This crate provides a generic ddisplay driver to connect to TFT displays
//! that implement the [MIPI DSI](https://www.mipi.org/specifications/dsi).
//! Currently only supports SPI with DC pin setups via the [display_interface]
//!
//! An optional batching of draws is supported via the `batch` feature (default on)
//!
//! ## Example
//! ```rust
//! // create a DisplayInterface from SPI and DC pin, with no manual CS control
//! let di = SPIInterfaceNoCS::new(spi, dc);
//! // create the ILI9486 display driver from the display interface and RST pin
//! let mut display = Display::ili9486(di, rst);
//! // clear the display to black
//! display.clear(Rgb666::BLACK)?;

pub mod instruction;

use crate::instruction::Instruction;

use display_interface::DataFormat;
use display_interface::DisplayError;
use display_interface::WriteOnlyDataCommand;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;

pub mod models;
use models::Model;

mod graphics;
mod no_pin;
pub use no_pin::*;

#[cfg(feature = "batch")]
mod batch;

///
/// Display driver to connect to TFT displays.
///
pub struct Display<DI, RST, MODEL>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
    MODEL: Model,
{
    // Display interface
    di: DI,
    // Reset pin.
    rst: Option<RST>,
    // Model
    model: MODEL,
    // Current orientation
    orientation: Orientation,
    // Current MADCTL value
    madctl: u8,
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

///
/// Options for displays used on initialization
///
#[derive(Debug, Default, Clone, Copy)]
pub struct DisplayOptions {
    /// Initial display orientation (without inverts)
    pub orientation: Orientation,
    /// Set to make display vertical refresh bottom to top
    pub invert_vertical_refresh: bool,
    /// Specify display color ordering
    pub color_order: ColorOrder,
    /// Set to make display horizontal refresh right to left
    pub invert_horizontal_refresh: bool,
}

impl DisplayOptions {
    /// Returns MADCTL register value for given display options
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
}

///
/// An error holding its source (pins or SPI)
///
#[derive(Debug)]
pub enum Error<PE> {
    DisplayError,
    Pin(PE),
}

impl<PE> From<DisplayError> for Error<PE> {
    fn from(_: DisplayError) -> Self {
        Error::DisplayError
    }
}

impl<DI, RST, M> Display<DI, RST, M>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
    M: Model,
{
    ///
    /// Creates a new [Display] driver instance with given [Model]
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `model` - the display [Model]
    ///
    pub fn with_model(di: DI, rst: Option<RST>, model: M) -> Self {
        Self {
            di,
            rst,
            model,
            orientation: Orientation::default(),
            madctl: 0,
        }
    }

    ///
    /// Runs commands to initialize the display
    ///
    /// # Arguments
    ///
    /// * `delay_source` - mutable reference to a [DelayUs] provider
    ///
    pub fn init(
        &mut self,
        delay_source: &mut impl DelayUs<u32>,
        options: DisplayOptions,
    ) -> Result<(), Error<RST::Error>> {
        self.madctl = self
            .model
            .init(&mut self.di, &mut self.rst, delay_source, options)?;
        self.orientation = options.orientation;
        Ok(())
    }

    ///
    /// Returns currently set [Orientation]
    ///
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    ///
    /// Sets display [Orientation]
    ///
    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), Error<RST::Error>> {
        let value = (self.madctl & 0b0001_1111) | orientation.value_u8();
        self.write_command(Instruction::MADCTL)?;
        self.write_data(&[value])?;
        self.orientation = orientation;
        self.madctl = value;
        Ok(())
    }

    ///
    /// Sets a pixel color at the given coords.
    ///
    /// # Arguments
    ///
    /// * `x` - x coordinate
    /// * `y` - y coordinate
    /// * `color` - the color value in pixel format of the display [Model]
    ///
    pub fn set_pixel(
        &mut self,
        x: u16,
        y: u16,
        color: M::ColorFormat,
    ) -> Result<(), Error<RST::Error>> {
        self.set_address_window(x, y, x, y)?;
        self.model
            .write_pixels(&mut self.di, core::iter::once(color))?;

        Ok(())
    }

    ///
    /// Sets pixel colors in given rectangle bounds.
    ///
    /// # Arguments
    ///
    /// * `sx` - x coordinate start
    /// * `sy` - y coordinate start
    /// * `ex` - x coordinate end
    /// * `ey` - y coordinate end
    /// * `colors` - anything that can provide `IntoIterator<Item = u16>` to iterate over pixel data
    ///
    pub fn set_pixels<T>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: T,
    ) -> Result<(), Error<RST::Error>>
    where
        T: IntoIterator<Item = M::ColorFormat>,
    {
        self.set_address_window(sx, sy, ex, ey)?;
        self.model.write_pixels(&mut self.di, colors)?;

        Ok(())
    }

    ///
    /// Sets scroll region
    /// # Arguments
    ///
    /// * `tfa` - Top fixed area
    /// * `vsa` - Vertical scrolling area
    /// * `bfa` - Bottom fixed area
    ///
    pub fn set_scroll_region(
        &mut self,
        tfa: u16,
        vsa: u16,
        bfa: u16,
    ) -> Result<(), Error<RST::Error>> {
        self.write_command(Instruction::VSCRDER)?;
        self.write_data(&tfa.to_be_bytes())?;
        self.write_data(&vsa.to_be_bytes())?;
        self.write_data(&bfa.to_be_bytes())?;

        Ok(())
    }

    ///
    /// Sets scroll offset "shifting" the displayed picture
    /// # Arguments
    ///
    /// * `offset` - scroll offset in pixels
    ///
    pub fn set_scroll_offset(&mut self, offset: u16) -> Result<(), Error<RST::Error>> {
        self.write_command(Instruction::VSCAD)?;
        self.write_data(&offset.to_be_bytes())
    }

    ///
    /// Release resources allocated to this driver back.
    /// This returns the display interface and the RST pin deconstructing the driver.
    ///
    pub fn release(self) -> (DI, Option<RST>, M) {
        (self.di, self.rst, self.model)
    }

    fn write_command(&mut self, command: Instruction) -> Result<(), Error<RST::Error>> {
        self.di
            .send_commands(DataFormat::U8(&[command as u8]))
            .map_err(|_| Error::DisplayError)?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), Error<RST::Error>> {
        self.di
            .send_data(DataFormat::U8(data))
            .map_err(|_| Error::DisplayError)
    }

    // Sets the address window for the display.
    fn set_address_window(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), Error<RST::Error>> {
        self.write_command(Instruction::CASET)?;
        self.write_data(&sx.to_be_bytes())?;
        self.write_data(&ex.to_be_bytes())?;
        self.write_command(Instruction::RASET)?;
        self.write_data(&sy.to_be_bytes())?;
        self.write_data(&ey.to_be_bytes())
    }

    ///
    /// Configures the tearing effect output.
    ///
    pub fn set_tearing_effect(
        &mut self,
        tearing_effect: TearingEffect,
    ) -> Result<(), Error<RST::Error>> {
        match tearing_effect {
            TearingEffect::Off => self.write_command(Instruction::TEOFF),
            TearingEffect::Vertical => {
                self.write_command(Instruction::TEON)?;
                self.write_data(&[0])
            }
            TearingEffect::HorizontalAndVertical => {
                self.write_command(Instruction::TEON)?;
                self.write_data(&[1])
            }
        }
    }
}
