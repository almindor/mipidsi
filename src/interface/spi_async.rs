use embassy_futures::block_on;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

use super::{FlushingInterface, Interface, InterfaceKind, SpiError};

/// Async Spi interface, including a dma buffer
///
/// The buffer should be a DMA buffer and is used to gather batches of pixel data to be sent over SPI.
/// The buffer should be large enough to accommodate the framebuffer size of the Display.
pub struct SpiInterfaceAsync<'a, SPI, DC> {
    spi: SPI,
    dc: DC,
    buffer: &'a mut [u8],
    index: usize,
}

impl<'a, SPI, DC> SpiInterfaceAsync<'a, SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    /// Create new interface
    pub fn new(spi: SPI, dc: DC, buffer: &'a mut [u8]) -> Self {
        // TODO: we should probably do at least an assertion of basic size requirement for the
        // buffer here.
        Self {
            spi,
            dc,
            buffer,
            index: 0,
        }
    }
}

impl<SPI, DC> Interface for SpiInterfaceAsync<'_, SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    type Word = u8;
    type Error = SpiError<SPI::Error, DC::Error>;

    const KIND: InterfaceKind = InterfaceKind::Serial4Line;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(SpiError::Dc)?;
        block_on(self.spi.write(&[command])).map_err(SpiError::Spi)?;
        self.dc.set_high().map_err(SpiError::Dc)?;
        block_on(self.spi.write(args)).map_err(SpiError::Spi)?;

        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        let mut arrays = pixels.into_iter();

        for chunk in self.buffer.chunks_exact_mut(N) {
            if let Some(array) = arrays.next() {
                let chunk: &mut [u8; N] = chunk.try_into().unwrap();
                *chunk = array;
                self.index += N;
            } else {
                break;
            };
        }

        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::Word; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        let fill_count = core::cmp::min(count, (self.buffer.len() / N) as u32);
        let filled_len = fill_count as usize * N;

        for chunk in self.buffer[self.index..(self.index + filled_len)].chunks_exact_mut(N) {
            let chunk: &mut [u8; N] = chunk.try_into().unwrap();
            *chunk = pixel;
        }

        self.index += filled_len;

        Ok(())
    }
}

impl<SPI, DC> FlushingInterface for SpiInterfaceAsync<'_, SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.spi
            .write(&self.buffer[..self.index])
            .await
            .map_err(SpiError::Spi)?;

        self.index = 0;

        Ok(())
    }
}
