use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::Dimensions, primitives::Rectangle, Pixel,
};
use embedded_hal::digital::OutputPin;

use super::{nth_u32, take_u32, TakeSkip};
use crate::models::Model;
use crate::{error::Error, Display};
use display_interface::WriteOnlyDataCommand;

impl<DI, M, RST> DrawTarget for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    type Error = Error;
    type Color = M::ColorFormat;

    #[cfg(not(feature = "batch"))]
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

    #[cfg(feature = "batch")]
    fn draw_iter<T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = Pixel<Self::Color>>,
    {
        use crate::batch::DrawBatch;

        self.draw_batch(item)
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

        let count = area.size.width * area.size.height;
        let mut colors = take_u32(core::iter::repeat(color), count);

        let sx = area.top_left.x as u16;
        let sy = area.top_left.y as u16;
        let ex = bottom_right.x as u16;
        let ey = bottom_right.y as u16;
        self.set_pixels(sx, sy, ex, ey, &mut colors)
    }
}
