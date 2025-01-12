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
pub struct AsyncSpiInterface<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI: SpiDevice, DC: Wait> AsyncSpiInterface<SPI, DC> {
    /// Create new interface
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

impl<SPI: SpiDevice, DC: Wait> AsyncInterface for AsyncSpiInterface<SPI, DC> {
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
}
