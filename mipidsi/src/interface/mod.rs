//! Interface traits and implementations

/// Command interface
pub trait CommandInterface {
    /// Error type
    type Error: core::fmt::Debug;

    /// Send a command with optional parameters
    ///
    /// [`CommandInterface::flush`] must be called to ensure the data is actually sent
    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error>;

    /// Sends any remaining buffered data
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Pixel interface
pub trait PixelInterface<P: Copy>: CommandInterface {
    /// Send a single pixel
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    ///
    /// [`CommandInterface::flush`] must be called to ensure the data is actually sent
    fn send_pixel(&mut self, pixel: P) -> Result<(), Self::Error>;

    fn send_pixels(&mut self, pixels: impl IntoIterator<Item = P>) -> Result<(), Self::Error> {
        for pixel in pixels {
            self.send_pixel(pixel)?;
        }
        Ok(())
    }

    /// Send the same pixel value multiple times
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    ///
    /// [`CommandInterface::flush`] must be called to ensure the data is actually sent
    fn send_repeated_pixel(&mut self, pixel: P, count: u32) -> Result<(), Self::Error>;
}

mod spi;
pub use spi::*;

mod parallel;
pub use parallel::*;
