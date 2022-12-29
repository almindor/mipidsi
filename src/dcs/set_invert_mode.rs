use crate::{instruction::Instruction, ColorInversion, Error};

use super::DcsCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInvertMode(pub ColorInversion);

impl DcsCommand for SetInvertMode {
    fn instruction(&self) -> Instruction {
        match self.0 {
            ColorInversion::Normal => Instruction::INVOFF,
            ColorInversion::Inverted => Instruction::INVON,
        }
    }

    fn fill_params_buf(&self, _buffer: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_invert_mode_chooses_correct_instruction() -> Result<(), Error> {
        let ste = SetInvertMode(ColorInversion::Inverted);

        let mut buffer = [0u8; 0];
        assert_eq!(ste.instruction(), Instruction::INVON);
        assert_eq!(ste.fill_params_buf(&mut buffer)?, 0);

        Ok(())
    }
}
