//! Interface traits and implementations

mod spi;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666};
pub use spi::*;

mod parallel;
pub use parallel::*;

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
pub trait PixelInterface: CommandInterface {
    /// The native width of the interface
    ///
    /// In most cases this will be u8, except for larger parallel interfaces such as
    /// 16 bit (currently supported)
    /// or 9 or 18 bit (currently unsupported)
    type PixelWord: Copy;
    /// Send a sequence of pixels
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    ///
    /// [`CommandInterface::flush`] must be called to ensure the data is actually sent
    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::PixelWord; N]>,
    ) -> Result<(), Self::Error>;

    /// Send the same pixel value multiple times
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    ///
    /// [`CommandInterface::flush`] must be called to ensure the data is actually sent
    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::PixelWord; N],
        count: u32,
    ) -> Result<(), Self::Error>;
}

impl<T: CommandInterface> CommandInterface for &mut T {
    type Error = T::Error;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        T::send_command(self, command, args)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        T::flush(self)
    }
}

impl<T: PixelInterface> PixelInterface for &mut T {
    type PixelWord = T::PixelWord;

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::PixelWord; N]>,
    ) -> Result<(), Self::Error> {
        T::send_pixels(self, pixels)
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::PixelWord; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        T::send_repeated_pixel(self, pixel, count)
    }
}

fn rgb565_to_bytes(pixel: Rgb565) -> [u8; 2] {
    embedded_graphics_core::pixelcolor::raw::ToBytes::to_be_bytes(pixel)
}
fn rgb565_to_u16(pixel: Rgb565) -> [u16; 1] {
    [u16::from_ne_bytes(
        embedded_graphics_core::pixelcolor::raw::ToBytes::to_ne_bytes(pixel),
    )]
}
fn rgb666_to_bytes(pixel: Rgb666) -> [u8; 3] {
    embedded_graphics_core::pixelcolor::raw::ToBytes::to_be_bytes(pixel).map(|x| x << 2)
}

/// This is an implementation detail, it should not be implemented or used outside this crate
pub trait PixelFormat<Word> {
    // this should just be
    // const N: usize;
    // fn convert(self) -> [Word; Self::N];
    // but that doesn't work yet

    #[doc(hidden)]
    fn send_pixels<DCS: PixelInterface<PixelWord = Word>>(
        dcs: &mut DCS,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DCS::Error>;

    #[doc(hidden)]
    fn send_repeated_pixel<DCS: PixelInterface<PixelWord = Word>>(
        dcs: &mut DCS,
        pixel: Self,
        count: u32,
    ) -> Result<(), DCS::Error>;
}

impl PixelFormat<u8> for Rgb565 {
    fn send_pixels<DCS: PixelInterface<PixelWord = u8>>(
        dcs: &mut DCS,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DCS::Error> {
        dcs.send_pixels(pixels.into_iter().map(rgb565_to_bytes))
    }

    fn send_repeated_pixel<DCS: PixelInterface<PixelWord = u8>>(
        dcs: &mut DCS,
        pixel: Self,
        count: u32,
    ) -> Result<(), DCS::Error> {
        dcs.send_repeated_pixel(rgb565_to_bytes(pixel), count)
    }
}

impl PixelFormat<u8> for Rgb666 {
    fn send_pixels<DCS: PixelInterface<PixelWord = u8>>(
        dcs: &mut DCS,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DCS::Error> {
        dcs.send_pixels(pixels.into_iter().map(rgb666_to_bytes))
    }

    fn send_repeated_pixel<DCS: PixelInterface<PixelWord = u8>>(
        dcs: &mut DCS,
        pixel: Self,
        count: u32,
    ) -> Result<(), DCS::Error> {
        dcs.send_repeated_pixel(rgb666_to_bytes(pixel), count)
    }
}

impl PixelFormat<u16> for Rgb565 {
    fn send_pixels<DCS: PixelInterface<PixelWord = u16>>(
        dcs: &mut DCS,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DCS::Error> {
        dcs.send_pixels(pixels.into_iter().map(rgb565_to_u16))
    }

    fn send_repeated_pixel<DCS: PixelInterface<PixelWord = u16>>(
        dcs: &mut DCS,
        pixel: Self,
        count: u32,
    ) -> Result<(), DCS::Error> {
        dcs.send_repeated_pixel(rgb565_to_u16(pixel), count)
    }
}
