use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::Dimensions, primitives::Rectangle, Pixel,
};
use embedded_hal::digital::OutputPin;

use super::take_u32;
use crate::{error::Error, models::Model, Display};
use display_interface::WriteOnlyDataCommand;

impl<DI, M, RST> DrawTarget for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    type Error = Error;
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
        super::fill_contiguous(self, area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        if let Some(ii) = super::calculate_intersection(area, &self.bounding_box())? {
            // we don't have buffer allowance so we have to use an iterator here
            let mut colors = take_u32(core::iter::repeat(color), ii.count);
            self.set_pixels(ii.sx, ii.sy, ii.ex, ii.ey, &mut colors)
        } else {
            Ok(())
        }
    }
}
