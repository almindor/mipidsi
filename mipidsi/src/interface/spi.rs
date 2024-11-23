use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
use embedded_hal::{digital::OutputPin, spi::SpiDevice};

use super::{CommandInterface, PixelInterface};

/// Spi interface error
#[derive(Clone, Copy, Debug)]
pub enum SpiError<SPI, DC> {
    /// SPI bus error
    Spi(SPI),
    /// Data/command pin error
    Dc(DC),
}

/// Spi interface
pub struct SpiInterface<'a, SPI, DC> {
    spi: SPI,
    dc: DC,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiDevice, DC: OutputPin> SpiInterface<'a, SPI, DC> {
    /// Create new interface
    pub fn new(spi: SPI, dc: DC, buffer: &'a mut [u8]) -> Self {
        Self { spi, dc, buffer }
    }

    fn push_bytes_repeated<const N: usize>(
        &mut self,
        bytes: [u8; N],
        count: u32,
    ) -> Result<(), SPI::Error> {
        let fill_count = core::cmp::min(count, (self.buffer.len() / N) as u32);
        let filled_len = fill_count as usize * N;
        for chunk in self.buffer[..(filled_len)].chunks_exact_mut(N) {
            let chunk: &mut [u8; N] = chunk.try_into().unwrap();
            *chunk = bytes;
        }

        let mut count = count;
        while count >= fill_count {
            self.spi.write(&self.buffer[..filled_len])?;
            count -= fill_count;
        }
        if count != 0 {
            self.spi
                .write(&self.buffer[..(count as usize * bytes.len())])?;
        }
        Ok(())
    }

    fn push_array_iter<const N: usize>(
        &mut self,
        arrays: impl IntoIterator<Item = [u8; N]>,
    ) -> Result<(), SPI::Error> {
        let mut arrays = arrays.into_iter();

        assert!(self.buffer.len() >= N);

        let mut done = false;
        while !done {
            let mut i = 0;
            for chunk in self.buffer.chunks_exact_mut(N) {
                if let Some(array) = arrays.next() {
                    let chunk: &mut [u8; N] = chunk.try_into().unwrap();
                    *chunk = array;
                    i += N;
                } else {
                    done = true;
                    break;
                };
            }
            self.spi.write(&self.buffer[..i])?;
        }
        Ok(())
    }
}

impl<SPI: SpiDevice, DC: OutputPin> CommandInterface for SpiInterface<'_, SPI, DC> {
    type Error = SpiError<SPI::Error, DC::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(SpiError::Dc)?;
        self.spi.write(&[command]).map_err(SpiError::Spi)?;
        self.dc.set_high().map_err(SpiError::Dc)?;
        self.spi.write(args).map_err(SpiError::Spi)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn rgb565_to_bytes(pixel: Rgb565) -> [u8; 2] {
    embedded_graphics_core::pixelcolor::raw::ToBytes::to_be_bytes(pixel)
}
fn rgb666_to_bytes(pixel: Rgb666) -> [u8; 3] {
    embedded_graphics_core::pixelcolor::raw::ToBytes::to_be_bytes(pixel).map(|x| x << 2)
}

impl<SPI: SpiDevice, DC: OutputPin> PixelInterface<Rgb565> for SpiInterface<'_, SPI, DC> {
    fn send_repeated_pixel(&mut self, pixel: Rgb565, count: u32) -> Result<(), Self::Error> {
        self.push_bytes_repeated(rgb565_to_bytes(pixel), count)
            .map_err(SpiError::Spi)
    }

    fn send_pixels(&mut self, pixels: impl IntoIterator<Item = Rgb565>) -> Result<(), Self::Error> {
        self.push_array_iter(pixels.into_iter().map(rgb565_to_bytes))
            .map_err(SpiError::Spi)
    }
}

impl<SPI: SpiDevice, DC: OutputPin> PixelInterface<Rgb666> for SpiInterface<'_, SPI, DC> {
    fn send_repeated_pixel(&mut self, pixel: Rgb666, count: u32) -> Result<(), Self::Error> {
        self.push_bytes_repeated(rgb666_to_bytes(pixel), count)
            .map_err(SpiError::Spi)
    }

    fn send_pixels(&mut self, pixels: impl IntoIterator<Item = Rgb666>) -> Result<(), Self::Error> {
        self.push_array_iter(pixels.into_iter().map(rgb666_to_bytes))
            .map_err(SpiError::Spi)
    }
}
