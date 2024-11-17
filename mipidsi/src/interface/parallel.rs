use embedded_graphics_core::pixelcolor::{raw::ToBytes, Rgb565};
use embedded_hal::digital::OutputPin;

use super::{CommandInterface, PixelInterface};

/// This trait represents the data pins of a parallel bus.
///
/// See [Generic8BitBus] and [Generic16BitBus] for generic implementations.
pub trait OutputBus {
    /// [u8] for 8-bit buses, [u16] for 16-bit buses, etc.
    type Word: Copy;

    /// Error type
    type Error: core::fmt::Debug;

    /// Set the output bus to a specific value
    fn set_value(&mut self, value: Self::Word) -> Result<(), Self::Error>;
}

macro_rules! generic_bus {
    ($GenericxBitBus:ident { type Word = $Word:ident; Pins {$($PX:ident => $x:tt,)*}}) => {
        /// A generic implementation of [OutputBus] using [OutputPin]s
        pub struct $GenericxBitBus<$($PX, )*> {
            pins: ($($PX, )*),
            last: Option<$Word>,
        }

        impl<$($PX, )*> $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin, )*
        {
            /// Creates a new bus. This does not change the state of the pins.
            ///
            /// The first pin in the tuple is the least significant bit.
            pub fn new(pins: ($($PX, )*)) -> Self {
                Self { pins, last: None }
            }

            /// Consumes the bus and returns the pins. This does not change the state of the pins.
            pub fn release(self) -> ($($PX, )*) {
                self.pins
            }
        }

        impl<$($PX, )* E> OutputBus
            for $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin<Error = E>, )*
            E: core::fmt::Debug,
        {
            type Word = $Word;
            type Error = E;

            fn set_value(&mut self, value: Self::Word) -> Result<(), Self::Error> {
                if self.last == Some(value) {
                    // It's quite common for multiple consecutive values to be identical, e.g. when filling or
                    // clearing the screen, so let's optimize for that case
                    return Ok(())
                }

                // Sets self.last to None.
                // We will update it to Some(value) *after* all the pins are succesfully set.
                let last = self.last.take();

                let changed = match last {
                    Some(old_value) => value ^ old_value,
                    None => !0, // all ones, this ensures that we will update all the pins
                };

                $(
                    let mask = 1 << $x;
                    if changed & mask != 0 {
                        if value & mask != 0 {
                            self.pins.$x.set_high()
                        } else {
                            self.pins.$x.set_low()
                        }
                        ?;
                    }
                )*

                self.last = Some(value);
                Ok(())
            }
        }

        impl<$($PX, )*> From<($($PX, )*)>
            for $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin, )*
        {
            fn from(pins: ($($PX, )*)) -> Self {
                Self::new(pins)
            }
        }
    };
}

generic_bus! {
    Generic8BitBus {
        type Word = u8;
        Pins {
            P0 => 0,
            P1 => 1,
            P2 => 2,
            P3 => 3,
            P4 => 4,
            P5 => 5,
            P6 => 6,
            P7 => 7,
        }
    }
}

generic_bus! {
    Generic16BitBus {
        type Word = u16;
        Pins {
            P0 => 0,
            P1 => 1,
            P2 => 2,
            P3 => 3,
            P4 => 4,
            P5 => 5,
            P6 => 6,
            P7 => 7,
            P8 => 8,
            P9 => 9,
            P10 => 10,
            P11 => 11,
            P12 => 12,
            P13 => 13,
            P14 => 14,
            P15 => 15,
        }
    }
}

/// Parallel interface error
#[derive(Clone, Copy, Debug)]
pub enum ParallelError<BUS, DC, WR> {
    /// Bus error
    Bus(BUS),
    /// Data/command pin error
    Dc(DC),
    /// Write pin error
    Wr(WR),
}

/// Parallel 8 Bit communication interface
///
/// This interface implements an 8-Bit "8080" style write-only display interface using any
/// 8-bit [OutputBus] implementation as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub struct PGpio8BitInterface<BUS, DC, WR> {
    bus: BUS,
    dc: DC,
    wr: WR,
}

impl<BUS, DC, WR> PGpio8BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u8>,
    DC: OutputPin,
    WR: OutputPin,
{
    /// Create new parallel GPIO interface for communication with a display driver
    pub fn new(bus: BUS, dc: DC, wr: WR) -> Self {
        Self { bus, dc, wr }
    }

    /// Consume the display interface and return
    /// the bus and GPIO pins used by it
    pub fn release(self) -> (BUS, DC, WR) {
        (self.bus, self.dc, self.wr)
    }

    fn send_byte(
        &mut self,
        byte: u8,
    ) -> Result<(), ParallelError<BUS::Error, DC::Error, WR::Error>> {
        self.wr.set_low().map_err(ParallelError::Wr)?;
        self.bus.set_value(byte).map_err(ParallelError::Bus)?;
        self.wr.set_high().map_err(ParallelError::Wr)
    }
}

impl<BUS, DC, WR> CommandInterface for PGpio8BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u8>,
    DC: OutputPin,
    WR: OutputPin,
{
    type Error = ParallelError<BUS::Error, DC::Error, WR::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(ParallelError::Dc)?;
        self.send_byte(command)?;
        self.dc.set_high().map_err(ParallelError::Dc)?;

        for arg in args {
            self.send_byte(*arg)?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<BUS, DC, WR> PixelInterface<Rgb565> for PGpio8BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u8>,
    DC: OutputPin,
    WR: OutputPin,
{
    fn send_pixel(&mut self, pixel: Rgb565) -> Result<(), Self::Error> {
        for byte in pixel.to_be_bytes() {
            self.send_byte(byte)?;
        }
        Ok(())
    }

    fn send_repeated_pixel(&mut self, pixel: Rgb565, count: u32) -> Result<(), Self::Error> {
        if count == 0 {
            return Ok(());
        }
        let [byte1, byte2] = pixel.to_be_bytes();
        if byte1 == byte2 {
            self.send_byte(byte1)?;
            for _ in 1..(count * 2) {
                self.wr.set_low().map_err(ParallelError::Wr)?;
                self.wr.set_high().map_err(ParallelError::Wr)?;
            }
            Ok(())
        } else {
            for _ in 0..count {
                self.send_pixel(pixel)?;
            }
            Ok(())
        }
    }
}

/// Parallel 16 Bit communication interface
///
/// This interface implements a 16-Bit "8080" style write-only display interface using any
/// 16-bit [OutputBus] implementation as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub struct PGpio16BitInterface<BUS, DC, WR> {
    bus: BUS,
    dc: DC,
    wr: WR,
}

impl<BUS, DC, WR> PGpio16BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u16>,
    DC: OutputPin,
    WR: OutputPin,
{
    /// Create new parallel GPIO interface for communication with a display driver
    pub fn new(bus: BUS, dc: DC, wr: WR) -> Self {
        Self { bus, dc, wr }
    }

    /// Consume the display interface and return
    /// the bus and GPIO pins used by it
    pub fn release(self) -> (BUS, DC, WR) {
        (self.bus, self.dc, self.wr)
    }

    fn send_word(
        &mut self,
        word: u16,
    ) -> Result<(), ParallelError<BUS::Error, DC::Error, WR::Error>> {
        self.wr.set_low().map_err(ParallelError::Wr)?;
        self.bus.set_value(word).map_err(ParallelError::Bus)?;
        self.wr.set_high().map_err(ParallelError::Wr)
    }
}

impl<BUS, DC, WR> CommandInterface for PGpio16BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u16>,
    DC: OutputPin,
    WR: OutputPin,
{
    type Error = ParallelError<BUS::Error, DC::Error, WR::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(ParallelError::Dc)?;
        self.send_word(u16::from(command))?;
        self.dc.set_high().map_err(ParallelError::Dc)?;

        for arg in args {
            self.send_word(u16::from(*arg))?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<BUS, DC, WR> PixelInterface<Rgb565> for PGpio16BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u16>,
    DC: OutputPin,
    WR: OutputPin,
{
    fn send_pixel(&mut self, pixel: Rgb565) -> Result<(), Self::Error> {
        self.send_word(u16::from_ne_bytes(pixel.to_ne_bytes()))
    }

    fn send_repeated_pixel(&mut self, pixel: Rgb565, count: u32) -> Result<(), Self::Error> {
        if count == 0 {
            return Ok(());
        }

        self.send_pixel(pixel)?;

        for _ in 1..count {
            self.wr.set_low().map_err(ParallelError::Wr)?;
            self.wr.set_high().map_err(ParallelError::Wr)?;
        }
        Ok(())
    }
}