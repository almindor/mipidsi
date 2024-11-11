use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::Dimensions, primitives::Rectangle, Pixel,
};
use embedded_hal::digital::OutputPin;

use crate::{batch::DrawBatch, dcs::WriteMemoryStart, error::Error, models::Model, Display};
use display_interface::{DataFormat, WriteOnlyDataCommand};

impl<DI, M, RST> DrawTarget for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    type Error = Error;
    type Color = M::ColorFormat;

    fn draw_iter<T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.draw_batch(item)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        super::fill_contiguous(self, area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        if let Some(ii) = super::calculate_intersection(area, &self.bounding_box())? {
            const BUFFER_SIZE: usize = 512;
            let mut raw_buf = [34u8; BUFFER_SIZE];
            let bytes_per_pixel = M::repeat_pixel_to_buffer(color, &mut raw_buf)?;

            // model does not support this yet
            if bytes_per_pixel == 0 {
                let mut colors = super::take_u32(core::iter::repeat(color), ii.count);
                return self.set_pixels(ii.sx, ii.sy, ii.ex, ii.ey, &mut colors);
            }

            self.set_address_window(ii.sx, ii.sy, ii.ex, ii.ey)?;

            self.dcs.write_command(WriteMemoryStart)?;

            let mut i = (ii.count as usize) * bytes_per_pixel;
            while i > 0 {
                let l = core::cmp::min(i, BUFFER_SIZE);
                self.dcs.di.send_data(DataFormat::U8(&raw_buf[0..l]))?;
                i -= l;
            }

            Ok(())
        } else {
            Ok(())
        }
    }
}
