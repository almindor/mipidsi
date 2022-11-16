//! Module for the CASET address window instruction constructors

use crate::{instruction::Instruction, Error};

use super::DcsCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caset(u16, u16);

impl Caset {
    ///
    /// Construct a new Caset range
    ///
    pub fn new(sx: u16, ex: u16) -> Self {
        Self(sx, ex)
    }
}

impl DcsCommand for Caset {
    fn instruction(&self) -> Instruction {
        Instruction::CASET
    }

    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let sx_bytes = self.0.to_be_bytes();
        let ex_bytes = self.1.to_be_bytes();
        buffer[0] = sx_bytes[0];
        buffer[1] = sx_bytes[1];
        buffer[2] = ex_bytes[0];
        buffer[3] = ex_bytes[1];

        Ok(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caset_fills_data_properly() -> Result<(), Error> {
        let caset = Caset::new(0, 320);

        let mut buffer = [0u8; 4];
        assert_eq!(caset.fill_params_buf(&mut buffer)?, 4);
        assert_eq!(buffer, [0, 0, 0x1, 0x40]);

        Ok(())
    }
}
