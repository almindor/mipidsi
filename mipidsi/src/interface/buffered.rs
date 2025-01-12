use super::Interface;

///
/// Interface with intermediate results and a flush method to apply them.
///
pub trait FlushingInterface: Interface {
    ///
    /// Applies changed in buffer to display
    ///
    fn flush(&mut self) -> Result<(), Self::Error>;
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
    // index of pixel bytes in buffer for one pixel, size of the pixel bytes data + count for repeat operation
    Repeat(usize, usize, u32),
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
    DI: Interface,
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
    DI: Interface<Word = u8>,
{
    type Word = u8;

    type Error = FlushingError<DI::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        if self.index + args.len() + 1 >= self.buffer.len() {
            return Err(FlushingError::BufferOverflow)
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
        let index = self.index;
        let mut bytes = 0usize;

        for pixel in pixels.into_iter().flatten() {
            if index + bytes >= self.buffer.len() {
                return Err(FlushingError::BufferOverflow)
            }

            self.buffer[index + bytes] = pixel;
            bytes += 1;
        }

        self.ops
            .push_front(Chunk::Pixels(index, bytes))
            .map_err(|_| FlushingError::MaxOperationsReached)?;

        self.index += bytes;

        Ok(())
    }

    fn send_pixels_from_buffer(
            &mut self,
            pixels: &[u8],
        ) -> Result<(), Self::Error> {
        self.di.send_pixels_from_buffer(pixels).map_err(|e| FlushingError::Underlaying(e))
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [u8; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        if self.index + N >= self.buffer.len() {
            return Err(FlushingError::BufferOverflow)
        }

        self.ops
            .push_front(Chunk::Repeat(self.index, N, count))
            .map_err(|_| FlushingError::MaxOperationsReached)?;

        self.buffer[self.index..self.index + N].copy_from_slice(&pixel);

        self.index += N;

        Ok(())
    }

    fn send_repeated_pixel_raw(
            &mut self,
            pixel_data: &[u8],
            count: u32,
        ) -> Result<(), Self::Error> {
        self.di.send_repeated_pixel_raw(pixel_data, count).map_err(|e| FlushingError::Underlaying(e))
    }
}

impl<DI, const MAX_OPS: usize> FlushingInterface for BufferedInterface<'_, DI, MAX_OPS>
where
    DI: Interface<Word = u8>, // TODO expand
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        while let Some(op) = self.ops.pop_back() {
            match op {
                Chunk::Command(index, arg_bytes) => self.di.send_command(
                    self.buffer[index],
                    &self.buffer[index + 1..index + 1 + arg_bytes],
                )?,
                Chunk::Pixels(index, bytes) => self.di.send_pixels_from_buffer(&self.buffer[index..index+bytes])?,
                Chunk::Repeat(index, bytes, count) => self.di.send_repeated_pixel_raw(&self.buffer[index..index+bytes], count)?,
            }
        }

        self.index = 0;

        Ok(())
    }
}
