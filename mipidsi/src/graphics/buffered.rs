use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::PixelColor,
    pixelcolor::{raw::ToBytes, Rgb555, Rgb565, Rgb666, Rgb888},
    primitives::Rectangle,
    Pixel,
};
use embedded_hal::digital::OutputPin;

use crate::{
    dcs::WriteMemoryStart,
    error::Error,
    models::{Endianness, Model},
    Display,
};
use display_interface::{DataFormat, WriteOnlyDataCommand};

const BUFFER_SIZE: usize = 512;

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
        let raw_color = match M::ENDIANNESS {
            Endianness::BigEndian => color.to_be_bytes(),
            Endianness::LittleEndian => color.to_le_bytes(),
        };
        fill_solid_specific_color(self, area, raw_color)
    }
}

impl<DI, M, RST> DrawTargetHelper<Rgb565> for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: Rgb565) -> Result<(), Error> {
        let raw_color = match M::ENDIANNESS {
            Endianness::BigEndian => color.to_be_bytes(),
            Endianness::LittleEndian => color.to_le_bytes(),
        };
        fill_solid_specific_color(self, area, raw_color)
    }
}

impl<DI, M, RST> DrawTargetHelper<Rgb666> for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: Rgb666) -> Result<(), Error> {
        let raw_color = match M::ENDIANNESS {
            Endianness::BigEndian => color.to_be_bytes(),
            Endianness::LittleEndian => color.to_le_bytes(),
        };
        fill_solid_specific_color(self, area, raw_color)
    }
}

impl<DI, M, RST> DrawTargetHelper<Rgb888> for Display<DI, M, RST>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    fn fill_solid_specific_color(&mut self, area: &Rectangle, color: Rgb888) -> Result<(), Error> {
        let raw_color = match M::ENDIANNESS {
            Endianness::BigEndian => color.to_be_bytes(),
            Endianness::LittleEndian => color.to_le_bytes(),
        };
        fill_solid_specific_color(self, area, raw_color)
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

    #[cfg(not(feature = "batch"))]
    fn draw_iter<T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in item {
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
        super::fill_contiguous(self, area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid_specific_color(area, color)
    }
}

// optimization helpers

fn fill_solid_specific_color<DI, M, RST, const N: usize>(
    display: &mut Display<DI, M, RST>,
    area: &Rectangle,
    raw_color: [u8; N],
) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
{
    let Some(ii) = display.calculate_fill_area(area) else {
        return Ok(());
    };
    let mut raw_buf = [0u8; BUFFER_SIZE];
    let pixels_per_buffer = fill_buffer(&mut raw_buf, raw_color);
    display.set_address_window(ii.sx, ii.sy, ii.ex, ii.ey)?;
    display.dcs.write_command(WriteMemoryStart)?;
    let mut pixel_count = usize::try_from(ii.count).unwrap();
    while pixel_count > pixels_per_buffer {
        display.dcs.di.send_data(DataFormat::U8(&raw_buf))?;
        pixel_count -= pixels_per_buffer;
    }
    if pixel_count > 0 {
        display
            .dcs
            .di
            .send_data(DataFormat::U8(&raw_buf[0..pixel_count]))?;
    }
    Ok(())
}

fn fill_buffer<const N: usize>(buffer: &mut [u8], data: [u8; N]) -> usize {
    for chunk in buffer.chunks_exact_mut(N) {
        chunk.copy_from_slice(&data);
    }
    buffer.len() / N
}
