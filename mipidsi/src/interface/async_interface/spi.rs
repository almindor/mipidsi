use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;
use embedded_hal_async::spi::SpiDevice as AsyncSpiDevice;

use super::AsyncInterface;

/// Spi interface error
#[derive(Clone, Copy, Debug)]
pub enum AsyncSpiError<SPI, DC> {
    /// SPI bus error
    Spi(SPI),
    /// Data/command pin error
    Dc(DC),
}

/// Hybrid Async/sync Spi interface that sends commands in a sync blocking way while
/// flushing any buffered (elsewhere) data using `flush`
///
pub struct AsyncSpiInterface<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI: SpiDevice + AsyncSpiDevice, DC: OutputPin> AsyncSpiInterface<SPI, DC> {
    /// Create new interface
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

impl<SPI: SpiDevice + AsyncSpiDevice, DC: OutputPin> AsyncInterface for AsyncSpiInterface<SPI, DC> {
    type Error = AsyncSpiError<SPI::Error, DC::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(AsyncSpiError::Dc)?;
        SpiDevice::write(&mut self.spi, &[command]).map_err(AsyncSpiError::Spi)?;
        self.dc.set_high().map_err(AsyncSpiError::Dc)?;

        SpiDevice::write(&mut self.spi, args).map_err(AsyncSpiError::Spi)
    }

    async fn send_pixels_from_buffer(&mut self, pixels: &[u8]) -> Result<(), Self::Error> {
        AsyncSpiDevice::write(&mut self.spi, pixels)
            .await
            .map_err(AsyncSpiError::Spi)
    }
}
