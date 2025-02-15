use embassy_futures::block_on;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

use crate::dcs::{DcsCommand, SetColumnAddress, SetPageAddress, WriteMemoryStart};

use super::{FlushingInterface, Interface, InterfaceKind, SpiError};

/// Async Spi interface, including a dma buffer
///
/// The buffer should be a DMA buffer and is used to gather batches of pixel data to be sent over SPI.
/// The buffer should be large enough to accommodate the `display_size` of the [crate::Display].
pub struct SpiInterfaceAsync<'a, SPI, DC> {
    spi: SPI,
    dc: DC,
    buffer: &'a mut [u8],
    // drawing window minmax values
    max_window_extents: WindowExtents,
    // last drawing window
    last_window_extents: WindowExtents,
    // display size
    display_size: (usize, usize),
    pixel_bytes: usize,
}

impl<'a, SPI, DC> SpiInterfaceAsync<'a, SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    /// Create new interface with given buffer and display size
    ///
    /// # Warning
    /// User must ensure that `buffer` size `display_size` correspond correctly.
    /// If `buffer` is smaller than `display_size` * "byte size of pixel" this interface
    /// will cause a panic during drawing operations.
    pub fn new(spi: SPI, dc: DC, buffer: &'a mut [u8], display_size: (usize, usize)) -> Self {
        Self {
            spi,
            dc,
            buffer,
            max_window_extents: WindowExtents::default_max(),
            last_window_extents: WindowExtents::default(),
            display_size,
            pixel_bytes: 0,
        }
    }

    fn send_command_inner(
        &mut self,
        command: u8,
        args: &[u8],
    ) -> Result<(), SpiError<SPI::Error, DC::Error>> {
        self.dc.set_low().map_err(SpiError::Dc)?;
        block_on(self.spi.write(&[command])).map_err(SpiError::Spi)?;
        self.dc.set_high().map_err(SpiError::Dc)?;
        block_on(self.spi.write(args)).map_err(SpiError::Spi)?;

        Ok(())
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
        // we must ensure drawing window sets are captured
        match command {
            // SetColumnAddress
            0x2A => {
                let sx = u16::from_be_bytes([args[0], args[1]]);
                let ex = u16::from_be_bytes([args[2], args[3]]);
                self.max_window_extents.apply_column_max(sx, ex);
                self.last_window_extents.x = (sx, ex);

                return Ok(());
            }
            // SetPageAddress
            0x2B => {
                let sy = u16::from_be_bytes([args[0], args[1]]);
                let ey = u16::from_be_bytes([args[2], args[3]]);
                self.max_window_extents.apply_page_max(sy, ey);
                self.last_window_extents.y = (sy, ey);

                return Ok(());
            }
            _ => {}
        }

        self.send_command_inner(command, args)
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        self.pixel_bytes = N; // TODO: refactor

        let mut arrays = pixels.into_iter();
        let mut lines = ExtentsRowIterator::default();

        while let Some((start_index, end_index)) = lines.next(
            &self.last_window_extents,
            self.display_size,
            self.pixel_bytes,
        ) {
            for chunk in self.buffer[start_index..end_index].chunks_exact_mut(N) {
                if let Some(array) = arrays.next() {
                    let chunk: &mut [u8; N] = chunk.try_into().unwrap();
                    *chunk = array;
                } else {
                    break;
                };
            }
        }

        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::Word; N],
        _count: u32,
    ) -> Result<(), Self::Error> {
        self.pixel_bytes = N; // TODO: refactor

        let mut lines = ExtentsRowIterator::default();
        while let Some((start_index, end_index)) = lines.next(
            &self.last_window_extents,
            self.display_size,
            self.pixel_bytes,
        ) {
            for chunk in self.buffer[start_index..end_index].chunks_exact_mut(N) {
                let chunk: &mut [u8; N] = chunk.try_into().unwrap();
                *chunk = pixel;
            }
        }

        Ok(())
    }
}

impl<SPI, DC> FlushingInterface for SpiInterfaceAsync<'_, SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        let mut param_bytes: [u8; 16] = [0; 16];

        // set drawing window based on max_window_extents
        let sca = SetColumnAddress::new(self.max_window_extents.x.0, self.max_window_extents.x.1);
        let n = sca.fill_params_buf(&mut param_bytes);
        self.send_command_inner(sca.instruction(), &param_bytes[..n])?;

        let spa = SetPageAddress::new(self.max_window_extents.y.0, self.max_window_extents.y.1);
        let n = spa.fill_params_buf(&mut param_bytes);
        self.send_command_inner(spa.instruction(), &param_bytes[..n])?;

        // draw area of max_window_extents from buffer
        self.send_command_inner(WriteMemoryStart.instruction(), &[])?;

        let mut lines = ExtentsRowIterator::default();
        while let Some((start_index, end_index)) = lines.next(
            &self.max_window_extents,
            self.display_size,
            self.pixel_bytes,
        ) {
            self.spi
                .write(&self.buffer[start_index..end_index])
                .await
                .map_err(SpiError::Spi)?;
        }

        // reset the max extents
        self.max_window_extents = WindowExtents::default_max();
        Ok(())
    }
}

#[derive(Debug, Default)]
struct WindowExtents {
    x: (u16, u16),
    y: (u16, u16),
}

impl WindowExtents {
    fn default_max() -> Self {
        Self {
            x: (u16::MAX, 0),
            y: (u16::MAX, 0),
        }
    }

    fn apply_column_max(&mut self, sx: u16, ex: u16) {
        self.x.0 = core::cmp::min(self.x.0, sx);
        self.x.1 = core::cmp::max(self.x.1, ex);
    }

    fn apply_page_max(&mut self, sy: u16, ey: u16) {
        self.y.0 = core::cmp::min(self.y.0, sy);
        self.y.1 = core::cmp::max(self.y.1, ey);
    }
}

#[derive(Default, Debug)]
struct ExtentsRowIterator(usize);

impl ExtentsRowIterator {
    pub fn next(
        &mut self,
        extents: &WindowExtents,
        display_size: (usize, usize),
        pixel_bytes: usize,
    ) -> Option<(usize, usize)> {
        if self.0 + extents.y.0 as usize > extents.y.1 as usize {
            return None;
        }

        // these are all in pixel counts
        let start_x = extents.x.0 as usize;
        let start_y = extents.y.0 as usize + self.0;
        let size_x = (extents.x.1 - extents.x.0 + 1) as usize;

        // these are in byte counts
        let start_index = (start_x + display_size.0 * start_y) * pixel_bytes;
        let end_index = start_index + (size_x * pixel_bytes);

        self.0 += 1;
        Some((start_index, end_index))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const PIXEL_BYTES: usize = 2;
    const DISPLAY_SIZE: (usize, usize) = (240, 130);

    #[test]
    fn test_zero_extents_row_iterator() {
        let extents = WindowExtents {
            x: (0, 0),
            y: (0, 0),
        };

        let mut iter = ExtentsRowIterator::default();

        if let Some((start_index, end_index)) = iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES) {
            assert_eq!(start_index, 0);
            assert_eq!(end_index, 2);
        } else {
            panic!("iterator did not return a value");
        }

        assert_eq!(iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES), None);
    }

    #[test]
    fn test_empty_extents_row_iterator() {
        let extents = WindowExtents {
            x: (44, 44),
            y: (33, 33),
        };

        let mut iter = ExtentsRowIterator::default();

        if let Some((start_index, end_index)) = iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES) {
            assert_eq!(start_index, (33 * DISPLAY_SIZE.0 + 44) * PIXEL_BYTES);
            assert_eq!(end_index, start_index + PIXEL_BYTES);
        } else {
            panic!("iterator did not return a value");
        }

        assert_eq!(iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES), None);
    }

    #[test]
    fn test_rect_extents_row_iterator() {
        let extents = WindowExtents {
            x: (55, 57),
            y: (33, 33),
        };

        let mut iter = ExtentsRowIterator::default();

        if let Some((start_index, end_index)) = iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES) {
            assert_eq!(start_index, (33 * DISPLAY_SIZE.0 + 55) * PIXEL_BYTES);
            assert_eq!(end_index, start_index + 3 * PIXEL_BYTES);
        } else {
            panic!("iterator did not return a value");
        }

        assert_eq!(iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES), None);
    }
}
