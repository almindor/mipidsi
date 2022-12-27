use crate::{instruction::Instruction, Error, TearingEffect};

use super::DcsCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTearingEffect(pub TearingEffect);

impl DcsCommand for SetTearingEffect {
    fn instruction(&self) -> Instruction {
        match self.0 {
            TearingEffect::Off => Instruction::TEOFF,
            TearingEffect::Vertical => Instruction::TEON,
            TearingEffect::HorizontalAndVertical => Instruction::TEON,
        }
    }

    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        match self.0 {
            TearingEffect::Off => Ok(0),
            TearingEffect::Vertical => {
                buffer[0] = 0x0;
                Ok(1)
            }
            TearingEffect::HorizontalAndVertical => {
                buffer[0] = 0x1;
                Ok(1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_tearing_effect_both_fills_param_properly() -> Result<(), Error> {
        let ste = SetTearingEffect(TearingEffect::HorizontalAndVertical);

        let mut buffer = [0u8; 1];
        assert_eq!(ste.fill_params_buf(&mut buffer)?, 1);
        assert_eq!(buffer, [0x1]);

        Ok(())
    }

    #[test]
    fn set_tearing_effect_off_fills_param_properly() -> Result<(), Error> {
        let ste = SetTearingEffect(TearingEffect::Off);

        let mut buffer = [0u8; 0];
        assert_eq!(ste.fill_params_buf(&mut buffer)?, 0);

        Ok(())
    }
}
