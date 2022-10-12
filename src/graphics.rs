use embedded_graphics_core::prelude::{DrawTarget, Point, Size};
use embedded_graphics_core::primitives::Rectangle;
use embedded_graphics_core::{prelude::OriginDimensions, Pixel};

use crate::models::Model;
use crate::{Display, Error};
use display_interface::WriteOnlyDataCommand;

impl<DI, M> DrawTarget for Display<DI, M>
where
    DI: WriteOnlyDataCommand,
    M: Model,
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
        if let Some(bottom_right) = area.bottom_right() {
            let mut count = 0u32;
            let max = area.size.width * area.size.height;

            let mut colors = colors.into_iter().take_while(|_| {
                count += 1;
                count <= max
            });

            let sx = area.top_left.x as u16;
            let sy = area.top_left.y as u16;
            let ex = bottom_right.x as u16;
            let ey = bottom_right.y as u16;
            self.set_pixels(sx, sy, ex, ey, &mut colors)
        } else {
            // nothing to draw
            Ok(())
        }
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let fb_size = self.model.options().framebuffer_size(self.orientation);
        let fb_rect = Rectangle::with_corners(
            Point::new(0, 0),
            Point::new(fb_size.0 as i32 - 1, fb_size.1 as i32 - 1),
        );
        let area = area.intersection(&fb_rect);

        if let Some(bottom_right) = area.bottom_right() {
            let mut count = 0u32;
            let max = area.size.width * area.size.height;

            let mut colors = core::iter::repeat(color).take_while(|_| {
                count += 1;
                count <= max
            });

            let sx = area.top_left.x as u16;
            let sy = area.top_left.y as u16;
            let ex = bottom_right.x as u16;
            let ey = bottom_right.y as u16;
            self.set_pixels(sx, sy, ex, ey, &mut colors)
        } else {
            // nothing to draw
            Ok(())
        }
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let fb_size = self.model.options().framebuffer_size(self.orientation);
        let pixel_count = usize::from(fb_size.0) * usize::from(fb_size.1);
        let colors = core::iter::repeat(color).take(pixel_count); // blank entire HW RAM contents
        self.set_pixels(0, 0, fb_size.0 - 1, fb_size.1 - 1, colors)
    }
}

impl<DI, MODEL> OriginDimensions for Display<DI, MODEL>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    fn size(&self) -> Size {
        let ds = self.model.options().display_size(self.orientation);
        let (width, height) = (u32::from(ds.0), u32::from(ds.1));
        Size::new(width, height)
    }
}
