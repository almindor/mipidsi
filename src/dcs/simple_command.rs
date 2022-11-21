//! Module for any "simple" [Instruction] implementing DcsCommand

use crate::{instruction::Instruction, ColorInversion, Error};

use super::DcsCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimpleCommand(Instruction);

impl SimpleCommand {
    ///
    /// Constructs a [Direct] [DcsCommand] from the given [Instruction]
    /// provided it's viable for no-params operations
    ///
    pub const fn new(instruction: Instruction) -> Self {
        match instruction {
            Instruction::SLPIN
            | Instruction::SLPOUT
            | Instruction::PTLON
            | Instruction::NORON
            | Instruction::DISPOFF
            | Instruction::DISPON
            | Instruction::IDLOFF
            | Instruction::IDLON
            | Instruction::INVOFF
            | Instruction::INVON
            | Instruction::RAMWR => Self(instruction),
            _ => panic!("Instruction not simple"),
        }
    }
}

impl DcsCommand for SimpleCommand {
    fn instruction(&self) -> Instruction {
        self.0
    }

    fn fill_params_buf(&self, _buffer: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

// implements DcsCommand for [ColorInversion]
impl DcsCommand for ColorInversion {
    fn instruction(&self) -> Instruction {
        match self {
            Self::Normal => Instruction::INVOFF,
            Self::Inverted => Instruction::INVON,
        }
    }

    fn fill_params_buf(&self, _buffer: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

impl Instruction {
    pub const fn to_command(self) -> SimpleCommand {
        SimpleCommand::new(self)
    }
}
