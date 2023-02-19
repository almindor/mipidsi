use crate::{ColorInversion, Error};

use super::DcsCommand;

/// Set Invert Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInvertMode(pub ColorInversion);

impl DcsCommand for SetInvertMode {
    fn instruction(&self) -> u8 {
        match self.0 {
            ColorInversion::Normal => 0x20,
            ColorInversion::Inverted => 0x21,
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
        assert_eq!(ste.instruction(), 0x21);
        assert_eq!(ste.fill_params_buf(&mut buffer)?, 0);

        Ok(())
    }
}
