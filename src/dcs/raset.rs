//! Module for the RASET address window instruction constructors

use crate::{instruction::Instruction, Error};

use super::DcsCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raset(u16, u16);

impl Raset {
    ///
    /// Construct a new Raset range
    ///
    pub fn new(sy: u16, ey: u16) -> Self {
        Self(sy, ey)
    }
}

impl DcsCommand for Raset {
    fn instruction(&self) -> Instruction {
        Instruction::CASET
    }

    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let sy_bytes = self.0.to_be_bytes();
        let ey_bytes = self.1.to_be_bytes();
        buffer[0] = sy_bytes[0];
        buffer[1] = sy_bytes[1];
        buffer[2] = ey_bytes[0];
        buffer[3] = ey_bytes[1];

        Ok(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raset_fills_data_properly() -> Result<(), Error> {
        let raset = Raset::new(0, 320);

        let mut buffer = [0u8; 4];
        assert_eq!(raset.fill_params_buf(&mut buffer)?, 4);
        assert_eq!(buffer, [0, 0, 0x1, 0x40]);

        Ok(())
    }
}
