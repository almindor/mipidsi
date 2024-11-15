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
    buffer: Buffer<'a>,
}

impl<'a, SPI: SpiDevice, DC: OutputPin> SpiInterface<'a, SPI, DC> {
    /// Create new interface
    pub fn new(spi: SPI, dc: DC, buffer: &'a mut [u8]) -> Self {
        let buffer = Buffer::new(buffer);
        Self { spi, dc, buffer }
    }
}

impl<'a, SPI: SpiDevice, DC: OutputPin> CommandInterface for SpiInterface<'a, SPI, DC> {
    type Error = SpiError<SPI::Error, DC::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.flush()?;
        self.dc.set_low().map_err(SpiError::Dc)?;
        self.spi.write(&[command]).map_err(SpiError::Spi)?;
        self.dc.set_high().map_err(SpiError::Dc)?;
        self.spi.write(args).map_err(SpiError::Spi)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.buffer
            .flush(|buf| self.spi.write(buf))
            .map_err(SpiError::Spi)
    }
}

fn rgb565_to_bytes(pixel: Rgb565) -> [u8; 2] {
    embedded_graphics_core::pixelcolor::raw::ToBytes::to_be_bytes(pixel)
}
fn rgb666_to_bytes(pixel: Rgb666) -> [u8; 3] {
    embedded_graphics_core::pixelcolor::raw::ToBytes::to_be_bytes(pixel).map(|x| x << 2)
}

impl<'a, SPI: SpiDevice, DC: OutputPin> PixelInterface<Rgb565> for SpiInterface<'a, SPI, DC> {
    fn send_pixel(&mut self, pixel: Rgb565) -> Result<(), Self::Error> {
        self.buffer
            .push_bytes(rgb565_to_bytes(pixel), |buf| self.spi.write(buf))
            .map_err(SpiError::Spi)
    }

    fn send_repeated_pixel(&mut self, pixel: Rgb565, count: u32) -> Result<(), Self::Error> {
        self.buffer
            .push_repeated_bytes(rgb565_to_bytes(pixel), count, |buf| self.spi.write(buf))
            .map_err(SpiError::Spi)
    }
}

impl<'a, SPI: SpiDevice, DC: OutputPin> PixelInterface<Rgb666> for SpiInterface<'a, SPI, DC> {
    fn send_pixel(&mut self, pixel: Rgb666) -> Result<(), Self::Error> {
        self.buffer
            .push_bytes(rgb666_to_bytes(pixel), |buf| self.spi.write(buf))
            .map_err(SpiError::Spi)
    }

    fn send_repeated_pixel(&mut self, pixel: Rgb666, count: u32) -> Result<(), Self::Error> {
        self.buffer
            .push_repeated_bytes(rgb666_to_bytes(pixel), count, |buf| self.spi.write(buf))
            .map_err(SpiError::Spi)
    }
}

struct Buffer<'a> {
    bytes: &'a mut [u8],
    index: usize,
}

impl<'a> Buffer<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            bytes: buffer,
            index: 0,
        }
    }

    fn flush<E>(&mut self, mut on_full: impl FnMut(&[u8]) -> Result<(), E>) -> Result<(), E> {
        let index = core::mem::replace(&mut self.index, 0);
        if index != 0 {
            on_full(&self.bytes[0..index])?;
        }
        Ok(())
    }

    fn push_bytes<const N: usize, E>(
        &mut self,
        pixel: [u8; N],
        on_full: impl FnMut(&[u8]) -> Result<(), E>,
    ) -> Result<(), E> {
        if self.bytes.len() - self.index < N {
            self.flush(on_full)?;
        }

        self.bytes[self.index..][..N].copy_from_slice(&pixel);
        self.index += N;
        Ok(())
    }

    fn push_repeated_bytes<const N: usize, E>(
        &mut self,
        pixel: [u8; N],
        mut count: u32,
        mut on_full: impl FnMut(&[u8]) -> Result<(), E>,
    ) -> Result<(), E> {
        if let Ok(count_bytes) = usize::try_from(count * N as u32) {
            if count_bytes >= self.bytes.len() - self.index {
                // There is enough remaining space in the buffer for all the new data
                for i in 0..count as usize {
                    self.bytes[(self.index + (i * N))..][..N].copy_from_slice(&pixel);
                }
                self.index += count_bytes;
                return Ok(());
            }
        }

        self.flush(&mut on_full)?;

        let buffer_len = self.bytes.len() / N;

        let fill_count = core::cmp::min(count as usize, buffer_len);

        for i in 0..fill_count {
            self.bytes[i * N..][..N].copy_from_slice(&pixel);
        }

        while count >= buffer_len as u32 {
            on_full(&self.bytes[0..(buffer_len * N)])?;

            count -= buffer_len as u32;
        }

        self.index = count as usize * 2;

        Ok(())
    }
}
