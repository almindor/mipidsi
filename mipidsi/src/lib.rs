#![no_std]
// associated re-typing not supported in rust yet
#![allow(clippy::type_complexity)]
#![warn(missing_docs)]

//! This crate provides a generic display driver to connect to TFT displays
//! that implement the [MIPI Display Command Set](https://www.mipi.org/specifications/display-command-set).
//!
//! Uses [display_interface](https://crates.io/crates/display-interface) to talk to the hardware via transports.
//!
//! An optional batching of draws is supported via the `batch` feature (default on)
//!
//! ### List of supported models
//!
//! * ST7789
//! * ST7735
//! * ILI9486
//! * ILI9341
//! * ILI9342C
//!
//! ## Examples
//! **For the ili9486 display, using the SPI interface with no chip select:**
//! ```rust ignore
//! use display_interface_spi::SPIInterfaceNoCS;    // Provides the builder for DisplayInterface
//! use mipidsi::Builder;                           // Provides the builder for Display
//! use embedded_graphics::pixelcolor::Rgb666;      // Provides the required color type
//!
//! /* Define the SPI interface as the variable `spi` */
//! /* Define the DC digital output pin as the variable `dc` */
//! /* Define the Reset digital output pin as the variable `rst` */
//!
//! // Create a DisplayInterface from SPI and DC pin, with no manual CS control
//! let di = SPIInterfaceNoCS::new(spi, dc);
//!
//! // Create the ILI9486 display driver from the display interface and optional RST pin
//! let mut display = Builder::ili9486(di)
//!     .init(&mut delay, Some(rst)).unwrap();
//!
//! // Clear the display to black
//! display.clear(Rgb666::BLACK).unwrap();
//! ```
//!
//! **For the ili9341 display, using the Parallel port, with the RGB666 color space and the Bgr
//! color order:**
//! ```rust ignore
//! // Provides the builder for DisplayInterface
//! use display_interface_parallel_gpio::{Generic8BitBus, PGPIO8BitInterface};
//! // Provides the builder for Display
//! use mipidsi::Builder;
//! // Provides the required color type
//! use embedded_graphics::pixelcolor::Rgb666;
//!
//! /* Define digital output pins d0 - d7 for the parallel port as `lcd_dX` */
//! /* Define the D/C digital output pin as `dc` */
//! /* Define the WR and Reset digital output pins with the initial state set as High as `wr` and
//! `rst` */
//!
//! // Create the DisplayInterface from a Generic8BitBus, which is made from the parallel pins
//! let bus = Generic8BitBus::new((lcd_d0, lcd_d1, lcd_d2,
//!     lcd_d3, lcd_d4, lcd_d5, lcd_d6, lcd_d7)).unwrap();
//! let di = PGPIO8BitInterface::new(bus, dc, wr);
//!
//! // Create the ILI9341 display driver from the display interface with the RGB666 color space
//! let mut display = Builder::ili9341_rgb666(di)
//!      .with_color_order(mipidsi::ColorOrder::Bgr)
//!      .init(&mut delay, Some(rst)).unwrap();
//!
//! // Clear the display to black
//! display.clear(Rgb666::RED).unwrap();
//! ```
//! Use the appropiate display interface crate for your needs:
//! - [`display-interface-spi`](https://docs.rs/display-interface-spi/)
//! - [`display-interface-parallel-gpio`](https://docs.rs/display-interface-parallel-gpio)
//! - [`display-interface-i2c`](https://docs.rs/display-interface-i2c/)
//!
//! ## Troubleshooting
//! See [document](https://github.com/almindor/mipidsi/blob/master/docs/TROUBLESHOOTING.md)

use dcs::Dcs;
use display_interface::WriteOnlyDataCommand;

pub mod error;
use embedded_hal::delay::DelayUs;
use embedded_hal::digital::OutputPin;
pub use error::Error;

pub mod options;
pub use options::*;

mod builder;
pub use builder::Builder;

pub mod dcs;

pub mod models;
use models::Model;

mod graphics;

mod test_image;
pub use test_image::TestImage;

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
    // DCS provider
    dcs: Dcs<DI>,
    // Model
    model: MODEL,
    // Reset pin
    rst: Option<RST>,
    // Model Options, includes current orientation
    options: ModelOptions,
    // Current MADCTL value copy for runtime updates
    madctl: dcs::SetAddressMode,
    // State monitor for sleeping TODO: refactor to a Model-connected state machine
    sleeping: bool,
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
    /// Sets display [Orientation] with mirror image parameter
    ///
    /// # Example
    /// ```rust ignore
    /// display.orientation(Orientation::Portrait(false)).unwrap();
    /// ```
    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), Error> {
        self.madctl = self.madctl.with_orientation(orientation); // set orientation
        self.dcs.write_command(self.madctl)?;

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
    /// # Example
    /// ```rust ignore
    /// display.set_pixel(100, 200, Rgb666::new(251, 188, 20)).unwrap();
    /// ```
    pub fn set_pixel(&mut self, x: u16, y: u16, color: M::ColorFormat) -> Result<(), Error> {
        self.set_address_window(x, y, x, y)?;
        self.model
            .write_pixels(&mut self.dcs, core::iter::once(color))?;

        Ok(())
    }

    ///
    /// Sets pixel colors in a rectangular region.
    ///
    /// The color values from the `colors` iterator will be drawn to the given region starting
    /// at the top left corner and continuing, row first, to the bottom right corner. No bounds
    /// checking is performed on the `colors` iterator and drawing will wrap around if the
    /// iterator returns more color values than the number of pixels in the given region.
    ///
    /// This is a low level function, which isn't intended to be used in regular user code.
    /// Consider using the [`fill_contiguous`](https://docs.rs/embedded-graphics/latest/embedded_graphics/draw_target/trait.DrawTarget.html#method.fill_contiguous)
    /// function from the `embedded-graphics` crate as an alternative instead.
    ///
    /// # Arguments
    ///
    /// * `sx` - x coordinate start
    /// * `sy` - y coordinate start
    /// * `ex` - x coordinate end
    /// * `ey` - y coordinate end
    /// * `colors` - anything that can provide `IntoIterator<Item = u16>` to iterate over pixel data
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
        self.model.write_pixels(&mut self.dcs, colors)?;

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
        let vscrdef = dcs::SetScrollArea::new(tfa, vsa, bfa);
        self.dcs.write_command(vscrdef)
    }

    ///
    /// Sets scroll offset "shifting" the displayed picture
    /// # Arguments
    ///
    /// * `offset` - scroll offset in pixels
    ///
    pub fn set_scroll_offset(&mut self, offset: u16) -> Result<(), Error> {
        let vscad = dcs::SetScrollStart::new(offset);
        self.dcs.write_command(vscad)
    }

    ///
    /// Release resources allocated to this driver back.
    /// This returns the display interface, reset pin and and the model deconstructing the driver.
    ///
    pub fn release(self) -> (DI, M, Option<RST>) {
        (self.dcs.release(), self.model, self.rst)
    }

    // Sets the address window for the display.
    fn set_address_window(&mut self, sx: u16, sy: u16, ex: u16, ey: u16) -> Result<(), Error> {
        // add clipping offsets if present
        let offset = self.options.window_offset();
        let (sx, sy, ex, ey) = (sx + offset.0, sy + offset.1, ex + offset.0, ey + offset.1);

        self.dcs.write_command(dcs::SetColumnAddress::new(sx, ex))?;
        self.dcs.write_command(dcs::SetPageAddress::new(sy, ey))
    }

    ///
    /// Configures the tearing effect output.
    ///
    pub fn set_tearing_effect(&mut self, tearing_effect: TearingEffect) -> Result<(), Error> {
        self.dcs
            .write_command(dcs::SetTearingEffect(tearing_effect))
    }

    ///
    /// Returns `true` if display is currently set to sleep.
    ///
    pub fn is_sleeping<D: DelayUs>(&self) -> bool {
        self.sleeping
    }

    ///
    /// Puts the display to sleep, reducing power consumption.
    /// Need to call [Self::wake] before issuing other commands
    ///
    pub fn sleep<D: DelayUs>(&mut self, delay: &mut D) -> Result<(), Error> {
        self.dcs.write_command(dcs::EnterSleepMode)?;
        // All supported models requires a 120ms delay before issuing other commands
        delay.delay_us(120_000);
        self.sleeping = true;
        Ok(())
    }

    ///
    /// Wakes the display after it's been set to sleep via [Self::sleep]
    ///
    pub fn wake<D: DelayUs>(&mut self, delay: &mut D) -> Result<(), Error> {
        self.dcs.write_command(dcs::ExitSleepMode)?;
        // ST7789 and st7735s have the highest minimal delay of 120ms
        delay.delay_us(120_000);
        self.sleeping = false;
        Ok(())
    }
}
