use super::{Interface, InterfacePixelFormat};
use crate::{models::Model, Display};
use core::future::Future;

mod spi;
use embedded_hal::digital::OutputPin;
pub use spi::*;

///
/// Interface with intermediate results and a flush method to apply them.
///
pub trait FlushingInterface: Interface {
    ///
    /// Applies changed in buffer to display
    ///
    fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Async version of the interface with expectation of [u8] data
pub trait AsyncInterface {
    /// Associated error
    type Error: core::fmt::Debug;

    /// Send a command with optional parameters
    fn send_command(
        &mut self,
        command: u8,
        args: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>>;

    /// Send raw pixel data from a &[u8] slice.
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    fn send_pixels_from_buffer(
        &mut self,
        pixels: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>>;
}

///
/// Error wrapper for [FlushingInterface] with differentiation between
/// underlaying errors on the internal [Interface] and maximum operations limit being
/// reached.
///
#[derive(Debug)]
pub enum FlushingError<E> {
    /// Unerlaying error reported from the internal [Interface]
    Underlaying(E),
    /// Maximum number of operations reached
    MaxOperationsReached,
    /// Buffer overflow
    BufferOverflow,
}

impl<E> From<E> for FlushingError<E> {
    fn from(value: E) -> Self {
        FlushingError::Underlaying(value)
    }
}

// Operation and byte index/size information for each
#[derive(Debug)]
enum Chunk {
    // index of command byte in buffer + size of argument bytes following it
    Command(usize, usize),
    // index of pixel bytes in buffer + their byte size
    Pixels(usize, usize),
}

///
/// Interface that uses user provided buffer to store operations data
/// that will be sent to the display.
///
pub struct BufferedInterface<'buffer, DI, const MAX_OPS: usize> {
    di: DI,
    buffer: &'buffer mut [u8],
    index: usize,
    ops: heapless::Deque<Chunk, MAX_OPS>,
}

impl<'buffer, DI, const MAX_OPS: usize> BufferedInterface<'buffer, DI, MAX_OPS>
where
    DI: AsyncInterface,
{
    ///
    /// Create new [BufferedInterface] with a given [Interface] to send buffer
    /// contents to the display and user provided &[u8] buffer to store them.
    ///
    pub fn new(di: DI, buffer: &'buffer mut [u8]) -> Self {
        Self {
            di,
            buffer,
            index: 0,
            ops: heapless::Deque::new(),
        }
    }
}

impl<DI, const MAX_OPS: usize> Interface for BufferedInterface<'_, DI, MAX_OPS>
where
    DI: AsyncInterface,
{
    type Word = u8;

    type Error = FlushingError<DI::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        if self.index + args.len() + 1 >= self.buffer.len() {
            return Err(FlushingError::BufferOverflow);
        }

        self.ops
            .push_front(Chunk::Command(self.index, args.len()))
            .map_err(|_| FlushingError::MaxOperationsReached)?;

        self.buffer[self.index] = command;
        self.buffer[self.index + 1..self.index + 1 + args.len()].copy_from_slice(args);

        self.index += args.len() + 1;

        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [u8; N]>,
    ) -> Result<(), Self::Error> {
        let mut bytes = 0usize;

        for pixel in pixels.into_iter().flatten() {
            if self.index + bytes >= self.buffer.len() {
                return Err(FlushingError::BufferOverflow);
            }

            self.buffer[self.index + bytes] = pixel;
            bytes += 1;
        }

        self.ops
            .push_front(Chunk::Pixels(self.index, bytes))
            .map_err(|_| FlushingError::MaxOperationsReached)?;

        self.index += bytes;

        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [u8; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        let n = N * (count as usize); // TODO: fix for u16 usize platforms

        if self.index + n >= self.buffer.len() {
            return Err(FlushingError::BufferOverflow);
        }

        self.ops
            .push_front(Chunk::Pixels(self.index, n))
            .map_err(|_| FlushingError::MaxOperationsReached)?;

        let mut i = 0;
        for _ in 0..count {
            self.buffer[i..i + N].copy_from_slice(&pixel);
            i += N;
        }

        self.index += n;

        Ok(())
    }
}

impl<DI, const MAX_OPS: usize> FlushingInterface for BufferedInterface<'_, DI, MAX_OPS>
where
    DI: AsyncInterface + Send,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        while let Some(op) = self.ops.pop_back() {
            match op {
                Chunk::Command(index, arg_bytes) => {
                    self.di
                        .send_command(
                            self.buffer[index],
                            &self.buffer[index + 1..index + 1 + arg_bytes],
                        )
                        .await?
                }
                Chunk::Pixels(index, bytes) => {
                    self.di
                        .send_pixels_from_buffer(&self.buffer[index..index + bytes])
                        .await?
                }
            }
        }

        self.index = 0;

        Ok(())
    }
}

impl<DI, M, RST> Display<DI, M, RST>
where
    DI: FlushingInterface,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    /// Write buffer to underlaying IO interface
    pub async fn flush(&mut self) -> Result<(), DI::Error> {
        self.di.flush().await
    }
}
