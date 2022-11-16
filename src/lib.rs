#![no_std]
// associated re-typing not supported in rust yet
#![allow(clippy::type_complexity)]

//! This crate provides a generic ddisplay driver to connect to TFT displays
//! that implement the [MIPI DSI](https://www.mipi.org/specifications/dsi).
//! Currently only supports SPI with DC pin setups via the [display_interface]
//!
//! An optional batching of draws is supported via the `batch` feature (default on)
//!
//! ### List of supported models
//!
//! * ST7789
//! * ST7735
//! * ILI9486
//! * ILI9342C
//!
//! ## Example
//! ```rust
//! // create a DisplayInterface from SPI and DC pin, with no manual CS control
//! let di = SPIInterfaceNoCS::new(spi, dc);
//! // create the ILI9486 display driver from the display interface and optional RST pin
//! let mut display = Builder::ili9486(di)
//!     .init(&mut delay, Some(rst));
//! // clear the display to black
//! display.clear(Rgb666::BLACK)?;

pub mod instruction;

use crate::instruction::Instruction;

use dcs::Dcs;
use dcs::Madctl;
use display_interface::DataFormat;
use display_interface::WriteOnlyDataCommand;

pub mod error;
use embedded_hal::digital::v2::OutputPin;
pub use error::Error;

pub mod options;
pub use options::*;

pub mod builder;
pub use builder::Builder;

pub mod dcs;

pub mod models;
use models::Model;

mod graphics;

#[cfg(feature = "batch")]
mod batch;

///
/// Display driver to connect to TFT displays.
///
pub struct Display<DI, MODEL, RST>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
    RST: OutputPin,
{
    // Display interface
    di: DI,
    // Model
    model: MODEL,
    // Reset pin
    rst: Option<RST>,
    // Model Options, includes current orientation
    options: ModelOptions,
    // Current MADCTL value
    madctl: Madctl,
}

impl<DI, M, RST> Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    ///
    /// Returns currently set [Orientation]
    ///
    pub fn orientation(&self) -> Orientation {
        self.options.orientation()
    }

    ///
    /// Sets display [Orientation]
    ///
    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), Error> {
        let mut dcs = Dcs::write_only(&mut self.di);
        dcs.write_command(&self.madctl.orientation(orientation))?;

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
    pub fn set_pixel(&mut self, x: u16, y: u16, color: M::ColorFormat) -> Result<(), Error> {
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
    ) -> Result<(), Error>
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
    pub fn set_scroll_region(&mut self, tfa: u16, vsa: u16, bfa: u16) -> Result<(), Error> {
        self.write_command(Instruction::VSCRDEF)?;
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
    pub fn set_scroll_offset(&mut self, offset: u16) -> Result<(), Error> {
        self.write_command(Instruction::VSCAD)?;
        self.write_data(&offset.to_be_bytes())
    }

    ///
    /// Release resources allocated to this driver back.
    /// This returns the display interface, reset pin and and the model deconstructing the driver.
    ///
    pub fn release(self) -> (DI, M, Option<RST>) {
        (self.di, self.model, self.rst)
    }

    fn write_command(&mut self, command: Instruction) -> Result<(), Error> {
        self.di.send_commands(DataFormat::U8(&[command as u8]))
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), Error> {
        self.di.send_data(DataFormat::U8(data))
    }

    // Sets the address window for the display.
    fn set_address_window(&mut self, sx: u16, sy: u16, ex: u16, ey: u16) -> Result<(), Error> {
        // add clipping offsets if present
        let offset = self.options.window_offset();
        let (sx, sy, ex, ey) = (sx + offset.0, sy + offset.1, ex + offset.0, ey + offset.1);

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
    pub fn set_tearing_effect(&mut self, tearing_effect: TearingEffect) -> Result<(), Error> {
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
