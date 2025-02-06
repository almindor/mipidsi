#![no_std]
// associated re-typing not supported in rust yet
#![allow(clippy::type_complexity)]
use dcs::InterfaceExt;

pub mod interface;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

pub mod options;
use interface::InterfacePixelFormat;
use options::MemoryMapping;

mod builder;
pub use builder::*;

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
    DI: interface::AsyncInterface,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    // DCS provider
    di: DI,
    // Model
    model: MODEL,
    // Reset pin
    rst: Option<RST>,
    // Model Options, includes current orientation
    options: options::ModelOptions,
    // Current MADCTL value copy for runtime updates
    madctl: dcs::SetAddressMode,
    // State monitor for sleeping TODO: refactor to a Model-connected state machine
    sleeping: bool,
}

impl<DI, M, RST> Display<DI, M, RST>
where
    DI: interface::AsyncInterface,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    ///
    /// Returns currently set [options::Orientation]
    ///
    pub fn orientation(&self) -> options::Orientation {
        self.options.orientation
    }

    ///
    /// Sets display [options::Orientation] with mirror image parameter
    ///
    /// # Examples
    ///
    /// ```
    /// use mipidsi::options::{Orientation, Rotation};
    ///
    /// # let mut display = mipidsi::_mock::new_mock_display();
    /// display.set_orientation(Orientation::default().rotate(Rotation::Deg180)).unwrap();
    /// ```
    pub async fn set_orientation(
        &mut self,
        orientation: options::Orientation,
    ) -> Result<(), DI::Error> {
        self.madctl = self.madctl.with_orientation(orientation); // set orientation
        self.di.write_command(self.madctl).await?;

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
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// # let mut display = mipidsi::_mock::new_mock_display();
    /// display.set_pixel(100, 200, Rgb565::new(251, 188, 20)).unwrap();
    /// ```
    pub async fn set_pixel(
        &mut self,
        x: u16,
        y: u16,
        color: M::ColorFormat,
    ) -> Result<(), DI::Error> {
        self.set_pixels(x, y, x, y, core::iter::once(color)).await
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
    /// <div class="warning">
    ///
    /// The end values of the X and Y coordinate ranges are inclusive, and no
    /// bounds checking is performed on these values. Using out of range values
    /// (e.g., passing `320` instead of `319` for a 320 pixel wide display) will
    /// result in undefined behavior.
    ///
    /// </div>
    pub async fn set_pixels<T>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: T,
    ) -> Result<(), DI::Error>
    where
        T: IntoIterator<Item = M::ColorFormat>,
    {
        self.set_address_window(sx, sy, ex, ey).await?;

        self.di.write_command(dcs::WriteMemoryStart).await?;

        M::ColorFormat::send_pixels(&mut self.di, colors)
    }

    /// Sets the vertical scroll region.
    ///
    /// The `top_fixed_area` and `bottom_fixed_area` arguments can be used to
    /// define an area on the top and/or bottom of the display which won't be
    /// affected by scrolling.
    ///
    /// Note that this method is not affected by the current display orientation
    /// and will always scroll vertically relative to the default display
    /// orientation.
    ///
    /// The combined height of the fixed area must not larger than the
    /// height of the framebuffer height in the default orientation.
    ///
    /// After the scrolling region is defined the [`set_vertical_scroll_offset`](Self::set_vertical_scroll_offset) can be
    /// used to scroll the display.
    pub async fn set_vertical_scroll_region(
        &mut self,
        top_fixed_area: u16,
        bottom_fixed_area: u16,
    ) -> Result<(), DI::Error> {
        let rows = M::FRAMEBUFFER_SIZE.1;

        let vscrdef = if top_fixed_area + bottom_fixed_area > rows {
            dcs::SetScrollArea::new(rows, 0, 0)
        } else {
            dcs::SetScrollArea::new(
                top_fixed_area,
                rows - top_fixed_area - bottom_fixed_area,
                bottom_fixed_area,
            )
        };

        self.di.write_command(vscrdef).await
    }

    /// Sets the vertical scroll offset.
    ///
    /// Setting the vertical scroll offset shifts the vertical scroll region
    /// upwards by `offset` pixels.
    ///
    /// Use [`set_vertical_scroll_region`](Self::set_vertical_scroll_region) to setup the scroll region, before
    /// using this method.
    pub async fn set_vertical_scroll_offset(&mut self, offset: u16) -> Result<(), DI::Error> {
        let vscad = dcs::SetScrollStart::new(offset);
        self.di.write_command(vscad).await
    }

    ///
    /// Release resources allocated to this driver back.
    /// This returns the display interface, reset pin and and the model deconstructing the driver.
    ///
    pub fn release(self) -> (DI, M, Option<RST>) {
        (self.di, self.model, self.rst)
    }

    // Sets the address window for the display.
    async fn set_address_window(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), DI::Error> {
        // add clipping offsets if present
        let mut offset = self.options.display_offset;
        let mapping = MemoryMapping::from(self.options.orientation);
        if mapping.reverse_columns {
            offset.0 = M::FRAMEBUFFER_SIZE.0 - (self.options.display_size.0 + offset.0);
        }
        if mapping.reverse_rows {
            offset.1 = M::FRAMEBUFFER_SIZE.1 - (self.options.display_size.1 + offset.1);
        }
        if mapping.swap_rows_and_columns {
            offset = (offset.1, offset.0);
        }

        let (sx, sy, ex, ey) = (sx + offset.0, sy + offset.1, ex + offset.0, ey + offset.1);

        self.di
            .write_command(dcs::SetColumnAddress::new(sx, ex))
            .await?;
        self.di
            .write_command(dcs::SetPageAddress::new(sy, ey))
            .await
    }

    ///
    /// Configures the tearing effect output.
    ///
    pub async fn set_tearing_effect(
        &mut self,
        tearing_effect: options::TearingEffect,
    ) -> Result<(), DI::Error> {
        self.di
            .write_command(dcs::SetTearingEffect::new(tearing_effect))
            .await
    }

    ///
    /// Returns `true` if display is currently set to sleep.
    ///
    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    ///
    /// Puts the display to sleep, reducing power consumption.
    /// Need to call [Self::wake] before issuing other commands
    ///
    pub async fn sleep<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), DI::Error> {
        self.di.write_command(dcs::EnterSleepMode).await?;
        // All supported models requires a 120ms delay before issuing other commands
        delay.delay_us(120_000);
        self.sleeping = true;
        Ok(())
    }

    ///
    /// Wakes the display after it's been set to sleep via [Self::sleep]
    ///
    pub async fn wake<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), DI::Error> {
        self.di.write_command(dcs::ExitSleepMode).await?;
        // ST7789 and st7735s have the highest minimal delay of 120ms
        delay.delay_us(120_000);
        self.sleeping = false;
        Ok(())
    }

    /// Returns the DCS interface for sending raw commands.
    ///
    /// # Safety
    ///
    /// Sending raw commands to the controller can lead to undefined behaviour,
    /// because the rest of the code isn't aware of any state changes that were caused by sending raw commands.
    /// The user must ensure that the state of the controller isn't altered in a way that interferes with the normal
    /// operation of this crate.
    pub unsafe fn dcs(&mut self) -> &mut DI {
        &mut self.di
    }
}

impl<DI, M, RST> Display<DI, M, RST>
where
    DI: interface::FlushingInterface,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    /// Flushes the framebuffer to the underlaying display interface
    pub async fn flush(&mut self) -> Result<(), DI::Error> {
        self.di.flush().await
    }
}
