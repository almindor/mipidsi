use embedded_graphics_core::{
    prelude::{Dimensions, DrawTarget, OriginDimensions, Size},
    primitives::Rectangle,
    Pixel,
};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use crate::{
    dcs,
    framebuffer::{ExtentsRowIterator, WindowExtents},
    graphics::{nth_u32, take_u32, TakeSkip},
    interface::{InterfaceAsync, InterfacePixelFormat},
    models::Model,
    options::{self, ModelOptions, Orientation},
};

///
/// Async Display driver to connect to TFT displays.
///
pub struct DisplayAsync<'buffer, DI, MODEL, RST>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    // DCS provider
    pub(crate) di: DI,
    // Model
    pub(crate) model: MODEL,
    // Reset pin
    pub(crate) rst: Option<RST>,
    // Model Options, includes current orientation
    pub(crate) options: ModelOptions,
    // Current MADCTL value copy for runtime updates
    pub(crate) madctl: dcs::SetAddressMode,
    // State monitor for sleeping TODO: refactor to a Model-connected state machine
    pub(crate) sleeping: bool,
    // drawing window minmax values
    pub(crate) max_window_extents: WindowExtents,
    // last drawing window
    pub(crate) last_window_extents: WindowExtents,
    // framebuffer
    pub(crate) buffer: &'buffer mut [u8],
}

impl<DI, M, RST> DisplayAsync<'_, DI, M, RST>
where
    DI: InterfaceAsync,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    ///
    /// Returns currently set [options::Orientation]
    ///
    pub fn orientation(&self) -> Orientation {
        self.options.orientation
    }

    /// TODO
    pub async fn set_orientation(&mut self, orientation: Orientation) -> Result<(), DI::Error> {
        self.madctl = self.madctl.with_orientation(orientation); // set orientation
        self.di.write_command(self.madctl).await?;

        Ok(())
    }

    /// TODO
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

    /// TODO
    pub async fn set_vertical_scroll_offset(&mut self, offset: u16) -> Result<(), DI::Error> {
        let vscad = dcs::SetScrollStart::new(offset);
        self.di.write_command(vscad).await
    }

    /// TODO
    pub fn release(self) -> (DI, M, Option<RST>) {
        (self.di, self.model, self.rst)
    }

    /// TODO
    pub async fn set_tearing_effect(
        &mut self,
        tearing_effect: options::TearingEffect,
    ) -> Result<(), DI::Error> {
        self.di
            .write_command(dcs::SetTearingEffect::new(tearing_effect))
            .await
    }

    /// TODO
    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    /// TODO
    pub async fn sleep<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), DI::Error> {
        self.di.write_command(dcs::EnterSleepMode).await?;
        // All supported models requires a 120ms delay before issuing other commands
        delay.delay_us(120_000).await;
        self.sleeping = true;
        Ok(())
    }

    /// TODO
    pub async fn wake<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), DI::Error> {
        self.di.write_command(dcs::ExitSleepMode).await?;
        // ST7789 and st7735s have the highest minimal delay of 120ms
        delay.delay_us(120_000).await;
        self.sleeping = false;
        Ok(())
    }

    // Sets the addre
    fn store_address_window(&mut self, sx: u16, sy: u16, ex: u16, ey: u16) {
        self.last_window_extents.x = (sx, ex);
        self.last_window_extents.y = (sy, ey);
        self.max_window_extents.apply_column_max(sx, ex);
        self.max_window_extents.apply_page_max(sy, ey);
    }

    /// TODO
    pub fn set_pixel(&mut self, x: u16, y: u16, color: M::ColorFormat) -> Result<(), DI::Error> {
        self.set_pixels(x, y, x, y, core::iter::once(color))
    }

    /// TODO
    pub fn set_pixels<T>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        pixels: T,
    ) -> Result<(), DI::Error>
    where
        T: IntoIterator<Item = M::ColorFormat>,
    {
        self.store_address_window(sx, sy, ex, ey);

        let mut bytes = M::ColorFormat::pixels_to_bytes(pixels).into_iter();
        let mut lines = ExtentsRowIterator::default();

        while let Some((start_index, end_index)) = lines.next(
            &self.last_window_extents,
            self.options.display_size(),
            Self::pixel_bytes(),
        ) {
            for buf_byte in &mut self.buffer[start_index..end_index] {
                if let Some(byte) = bytes.next() {
                    *buf_byte = byte;
                } else {
                    break;
                };
            }
        }

        Ok(())
    }

    /// TODO
    pub async fn flush(&mut self) -> Result<(), DI::Error> {
        // set drawing window based on max_window_extents
        let sca =
            dcs::SetColumnAddress::new(self.max_window_extents.x.0, self.max_window_extents.x.1);
        self.di.write_command(sca).await?;

        let spa =
            dcs::SetPageAddress::new(self.max_window_extents.y.0, self.max_window_extents.y.1);

        self.di.write_command(spa).await?;

        // draw area of max_window_extents from buffer
        self.di.write_command(dcs::WriteMemoryStart).await?;

        let mut lines = ExtentsRowIterator::default();
        while let Some((start_index, end_index)) = lines.next(
            &self.max_window_extents,
            self.options.display_size(),
            Self::pixel_bytes(),
        ) {
            self.di
                .send_buffer(&self.buffer[start_index..end_index])
                .await?;
        }

        // reset the max extents
        self.max_window_extents = WindowExtents::default_max();
        Ok(())
    }

    const fn pixel_bytes() -> usize {
        2 // TODO!
    }
}

impl<DI, M, RST> DrawTarget for DisplayAsync<'_, DI, M, RST>
where
    DI: InterfaceAsync,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    type Error = DI::Error;
    type Color = M::ColorFormat;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let x = pixel.0.x as u16;
            let y = pixel.0.y as u16;

            self.set_pixel(x, y, pixel.1)?;
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let intersection = area.intersection(&self.bounding_box());
        let Some(bottom_right) = intersection.bottom_right() else {
            // No intersection -> nothing to draw
            return Ok(());
        };

        // Unchecked casting to u16 cannot fail here because the values are
        // clamped to the display size which always fits in an u16.
        let sx = intersection.top_left.x as u16;
        let sy = intersection.top_left.y as u16;
        let ex = bottom_right.x as u16;
        let ey = bottom_right.y as u16;

        let count = intersection.size.width * intersection.size.height;

        let mut colors = colors.into_iter();

        if &intersection == area {
            // Draw the original iterator if no edge overlaps the framebuffer
            self.set_pixels(sx, sy, ex, ey, take_u32(colors, count))
        } else {
            // Skip pixels above and to the left of the intersection
            let mut initial_skip = 0;
            if intersection.top_left.y > area.top_left.y {
                initial_skip += intersection.top_left.y.abs_diff(area.top_left.y) * area.size.width;
            }
            if intersection.top_left.x > area.top_left.x {
                initial_skip += intersection.top_left.x.abs_diff(area.top_left.x);
            }
            if initial_skip > 0 {
                nth_u32(&mut colors, initial_skip - 1);
            }

            // Draw only the pixels which don't overlap the edges of the framebuffer
            let take_per_row = intersection.size.width;
            let skip_per_row = area.size.width - intersection.size.width;
            self.set_pixels(
                sx,
                sy,
                ex,
                ey,
                take_u32(TakeSkip::new(colors, take_per_row, skip_per_row), count),
            )
        }
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box());
        let Some(bottom_right) = area.bottom_right() else {
            // No intersection -> nothing to draw
            return Ok(());
        };

        let count = (area.size.width * area.size.height) as usize;

        let sx = area.top_left.x as u16;
        let sy = area.top_left.y as u16;
        let ex = bottom_right.x as u16;
        let ey = bottom_right.y as u16;

        self.store_address_window(sx, sy, ex, ey);
        self.set_pixels(sx, sy, ex, ey, core::iter::repeat_n(color, count))
    }
}

impl<DI, MODEL, RST> OriginDimensions for DisplayAsync<'_, DI, MODEL, RST>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    fn size(&self) -> Size {
        let ds = self.options.display_size();
        let (width, height) = (u32::from(ds.0), u32::from(ds.1));
        Size::new(width, height)
    }
}
