//! Module for the COLMOD instruction constructors

use embedded_graphics_core::prelude::{PixelColor, RawData};

use crate::{instruction::Instruction, Error};

use super::DcsCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colmod(u8);

impl Colmod {
    ///
    /// Set the color format that the MCU will be sending to this
    /// display. Uses RawData::BITS_PER_PIXEL to decide.
    ///
    pub const fn new<PF>() -> Self
    where
        PF: PixelColor,
    {
        match PF::Raw::BITS_PER_PIXEL {
            16 => Self(0b0101_0101),
            18 => Self(0b0110_0110),
            24 => Self(0b0110_0111), // not sure if this is right
            _ => panic!("Incompatible PixelFormat size"),
        }
    }
}

impl DcsCommand for Colmod {
    fn instruction(&self) -> Instruction {
        Instruction::MADCTL
    }

    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        buffer[0] = self.0;
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use embedded_graphics_core::pixelcolor::*;

    use super::*;

    #[test]
    fn colmod_rgb565_is_16bit() -> Result<(), Error> {
        let colmod = Colmod::new::<Rgb565>();

        let mut bytes = [0u8];
        assert_eq!(colmod.fill_params_buf(&mut bytes)?, 1);
        assert_eq!(bytes, [0b0101_0101u8]);

        Ok(())
    }

    #[test]
    fn colmod_rgb666_is_18bit() -> Result<(), Error> {
        let colmod = Colmod::new::<Rgb666>();

        let mut bytes = [0u8];
        assert_eq!(colmod.fill_params_buf(&mut bytes)?, 1);
        assert_eq!(bytes, [0b0110_0110u8]);

        Ok(())
    }

    #[test]
    fn colmod_rgb888_is_24bit() -> Result<(), Error> {
        let colmod = Colmod::new::<Rgb888>();

        let mut bytes = [0u8];
        assert_eq!(colmod.fill_params_buf(&mut bytes)?, 1);
        assert_eq!(bytes, [0b0110_0111u8]);

        Ok(())
    }
}
