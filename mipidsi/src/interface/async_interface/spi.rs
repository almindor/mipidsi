use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use super::AsyncInterface;

/// Spi interface error
#[derive(Clone, Copy, Debug)]
pub enum AsyncSpiError<SPI, DC> {
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
/// You may want to use [static_cell](https://crates.io/crates/static_cell)
/// to obtain a `&'static mut [u8; N]` buffer.
pub struct AsyncSpiInterface<'a, SPI, DC> {
    spi: SPI,
    dc: DC,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiDevice, DC: Wait> AsyncSpiInterface<'a, SPI, DC> {
    /// Create new interface
    pub fn new(spi: SPI, dc: DC, buffer: &'a mut [u8]) -> Self {
        Self { spi, dc, buffer }
    }
}

impl<SPI: SpiDevice, DC: Wait> AsyncInterface for AsyncSpiInterface<'_, SPI, DC> {
    type Error = AsyncSpiError<SPI::Error, DC::Error>;

    async fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.wait_for_low().await.map_err(AsyncSpiError::Dc)?;
        self.spi
            .write(&[command])
            .await
            .map_err(AsyncSpiError::Spi)?;
        self.dc.wait_for_high().await.map_err(AsyncSpiError::Dc)?;

        self.spi.write(args).await.map_err(AsyncSpiError::Spi)
    }

    async fn send_pixels_from_buffer(&mut self, pixels: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(pixels).await.map_err(AsyncSpiError::Spi)
    }

    async fn send_repeated_pixel_raw(
        &mut self,
        pixel_data: &[u8],
        count: u32,
    ) -> Result<(), Self::Error> {
        let n = pixel_data.len();
        let fill_count = core::cmp::min(count, (self.buffer.len() / n) as u32);
        let filled_len = fill_count as usize * n;
        let mut i = 0;

        // TODO: optimize
        for _ in 0..fill_count {
            for byte in pixel_data {
                self.buffer[i] = *byte;
                i += 1;
            }
        }

        let mut count = count;
        while count >= fill_count {
            self.spi
                .write(&self.buffer[..filled_len])
                .await
                .map_err(AsyncSpiError::Spi)?;
            count -= fill_count;
        }
        if count != 0 {
            self.spi
                .write(&self.buffer[..(count as usize * n)])
                .await
                .map_err(AsyncSpiError::Spi)?;
        }

        Ok(())
    }
}
