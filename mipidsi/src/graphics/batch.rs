use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Dimensions,
    pixelcolor::PixelColor,
    pixelcolor::{raw::ToBytes, Rgb555, Rgb565, Rgb666, Rgb888},
    primitives::Rectangle,
    Pixel,
};
use embedded_hal::digital::OutputPin;

use crate::{
    batch::DrawBatch,
    dcs::WriteMemoryStart,
    error::Error,
    models::{Endianness, Model},
    Display,
};
use display_interface::{DataFormat, WriteOnlyDataCommand};

// used to get "specialization hack" going for PixelColor
trait DrawTargetHelper<C: PixelColor> {
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: C) -> Result<(), Error>;
}

impl<DI, M, RST> DrawTargetHelper<Rgb555> for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: Rgb555) -> Result<(), Error> {
        const BUFFER_SIZE: usize = 512;
        let mut raw_buf = [0u8; BUFFER_SIZE];

        fill_solid_specific_color(self, area, || {
            let bytes = match M::ENDIANNESS {
                Endianness::BigEndian => color.to_be_bytes(),
                Endianness::LittleEndian => color.to_le_bytes(),
            };
            (repeat_pixel_to_buffer_bytes(&bytes, &mut raw_buf), &raw_buf)
        })
    }
}

impl<DI, M, RST> DrawTargetHelper<Rgb565> for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: Rgb565) -> Result<(), Error> {
        const BUFFER_SIZE: usize = 512;
        let mut raw_buf = [0u8; BUFFER_SIZE];

        fill_solid_specific_color(self, area, || {
            let bytes = match M::ENDIANNESS {
                Endianness::BigEndian => color.to_be_bytes(),
                Endianness::LittleEndian => color.to_le_bytes(),
            };
            (repeat_pixel_to_buffer_bytes(&bytes, &mut raw_buf), &raw_buf)
        })
    }
}

impl<DI, M, RST> DrawTargetHelper<Rgb666> for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: Rgb666) -> Result<(), Error> {
        const BUFFER_SIZE: usize = 512;
        let mut raw_buf = [0u8; BUFFER_SIZE];

        fill_solid_specific_color(self, area, || {
            let bytes = match M::ENDIANNESS {
                Endianness::BigEndian => color.to_be_bytes(),
                Endianness::LittleEndian => color.to_le_bytes(),
            };
            (repeat_pixel_to_buffer_bytes(&bytes, &mut raw_buf), &raw_buf)
        })
    }
}

impl<DI, M, RST> DrawTargetHelper<Rgb888> for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: Rgb888) -> Result<(), Error> {
        const BUFFER_SIZE: usize = 512;
        let mut raw_buf = [0u8; BUFFER_SIZE];

        fill_solid_specific_color(self, area, || {
            let bytes = match M::ENDIANNESS {
                Endianness::BigEndian => color.to_be_bytes(),
                Endianness::LittleEndian => color.to_le_bytes(),
            };
            (repeat_pixel_to_buffer_bytes(&bytes, &mut raw_buf), &raw_buf)
        })
    }
}

impl<DI, M, RST> DrawTarget for Display<DI, M, RST>
where
    Self: DrawTargetHelper<M::ColorFormat>,
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
        self.fill_solid_specific_color(area, color)
    }
}

// optimization helpers

fn fill_solid_specific_color<'d, DI, M, RST, F>(
    display: &'d mut Display<DI, M, RST>,
    area: &Rectangle,
    make_buffer: F,
) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
    F: FnOnce() -> (usize, &'d [u8]),
{
    if let Some(ii) = super::calculate_intersection(area, &display.bounding_box())? {
        let (bytes_per_pixel, raw_buf) = make_buffer();

        display.set_address_window(ii.sx, ii.sy, ii.ex, ii.ey)?;

        display.dcs.write_command(WriteMemoryStart)?;

        let mut i = (ii.count as usize) * bytes_per_pixel;
        while i > 0 {
            let l = core::cmp::min(i, raw_buf.len());
            display.dcs.di.send_data(DataFormat::U8(&raw_buf[0..l]))?;
            i -= l;
        }

        Ok(())
    } else {
        Ok(())
    }
}

fn repeat_pixel_to_buffer_bytes(bytes: &[u8], buf: &mut [u8]) -> usize {
    let mut j = 0;
    for val in buf {
        *val = bytes[j];

        j += 1;
        if j >= bytes.len() {
            j = 0;
        }
    }

    bytes.len()
}
