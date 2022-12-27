//! Module for the CASET address window instruction constructors

use crate::{instruction::Instruction, Error};

use super::DcsCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caset {
    start_column: u16,
    end_column: u16,
}

impl Caset {
    ///
    /// Construct a new Caset range
    ///
    pub fn new(start_column: u16, end_column: u16) -> Self {
        Self {
            start_column,
            end_column,
        }
    }
}

impl DcsCommand for Caset {
    fn instruction(&self) -> Instruction {
        Instruction::CASET
    }

    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        buffer[0..2].copy_from_slice(&self.start_column.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.end_column.to_be_bytes());

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