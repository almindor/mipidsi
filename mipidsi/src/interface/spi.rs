use embedded_hal::{digital::OutputPin, spi::SpiDevice};

use super::Interface;

/// Spi interface error
#[derive(Clone, Copy, Debug)]
pub enum SpiError<SPI, DC> {
    /// SPI bus error
    Spi(SPI),
    /// Data/command pin error
    Dc(DC),
}

/// Spi interface, including a buffer
///
/// The buffer is used to gather batches of pixel data to be sent over SPI.
/// Larger buffers will genererally be faster (with diminishing returns), at the expense of using more RAM.
/// The buffer should be at least big enough to hold a few pixels of data.
///
/// The buffer can be any type that implements [`AsMut<[u8]>`](AsMut) such as
/// - Mutable slices, `&mut [u8]`
/// - Owned arrays, `[u8; N]`
/// - Or even heap types like `Box<[u8]>` or `Vec<u8>`
///
/// # Examples:
///
/// Slice buffer
/// ```rust
/// # use mipidsi::interface::SpiInterface;
/// # let spi = mipidsi::_mock::MockSpi;
/// # let dc = mipidsi::_mock::MockOutputPin;
/// let mut buffer = [0_u8; 128];
/// let iface = SpiInterface::new(spi, dc, &mut buffer);
/// ```
///
/// Array buffer
/// ```rust
/// # use mipidsi::interface::SpiInterface;
/// # let spi = mipidsi::_mock::MockSpi;
/// # let dc = mipidsi::_mock::MockOutputPin;
/// let iface = SpiInterface::new(spi, dc, [0_u8; 128]);
/// ```
pub struct SpiInterface<SPI, DC, B> {
    spi: SPI,
    dc: DC,
    buffer: B,
}

impl<SPI: SpiDevice, DC: OutputPin, B: AsMut<[u8]>> SpiInterface<SPI, DC, B> {
    /// Create new interface
    pub fn new(spi: SPI, dc: DC, buffer: B) -> Self {
        Self { spi, dc, buffer }
    }
}

impl<SPI: SpiDevice, DC: OutputPin, B: AsMut<[u8]>> Interface for SpiInterface<SPI, DC, B> {
    type Word = u8;
    type Error = SpiError<SPI::Error, DC::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(SpiError::Dc)?;
        self.spi.write(&[command]).map_err(SpiError::Spi)?;
        self.dc.set_high().map_err(SpiError::Dc)?;
        self.spi.write(args).map_err(SpiError::Spi)?;
        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        let mut arrays = pixels.into_iter();

        let buffer = self.buffer.as_mut();

        assert!(buffer.len() >= N);

        let mut done = false;
        while !done {
            let mut i = 0;
            for chunk in buffer.chunks_exact_mut(N) {
                if let Some(array) = arrays.next() {
                    let chunk: &mut [u8; N] = chunk.try_into().unwrap();
                    *chunk = array;
                    i += N;
                } else {
                    done = true;
                    break;
                };
            }
            self.spi.write(&buffer[..i]).map_err(SpiError::Spi)?;
        }
        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::Word; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        let buffer = self.buffer.as_mut();

        let fill_count = core::cmp::min(count, (buffer.len() / N) as u32);
        let filled_len = fill_count as usize * N;
        for chunk in buffer[..(filled_len)].chunks_exact_mut(N) {
            let chunk: &mut [u8; N] = chunk.try_into().unwrap();
            *chunk = pixel;
        }

        let mut count = count;
        while count >= fill_count {
            self.spi
                .write(&buffer[..filled_len])
                .map_err(SpiError::Spi)?;
            count -= fill_count;
        }
        if count != 0 {
            self.spi
                .write(&buffer[..(count as usize * pixel.len())])
                .map_err(SpiError::Spi)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
