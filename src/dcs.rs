use display_interface::{WriteOnlyDataCommand, DataFormat};

use crate::{Error, instruction::Instruction};

mod madctl;
use madctl::*;


///
/// Provides a constructor for complex commands
/// e.g. `Madctl::new().with_bgr(true).bytes()`
/// 
pub trait DcsCommand {
    fn instruction(&self) -> Instruction;
    fn bytes(&self) -> &[u8];
}

///
/// Representation of the MIPI Display Command Set
/// Allows calling commands as methods with builder pattern
/// for the more complicated ones. Any command can be executed directly using the [Dcs::write_command] method.
/// Raw instructions can be sent using [Dcs::write_instruction].
/// Display interface can be accessed directly for data transfers using the `di` public field.
/// 
struct Dcs<DI> {
    /// 
    /// Display interface instance
    /// 
    pub di: DI,
}

impl<DI> Dcs<DI>
where
    DI: WriteOnlyDataCommand
{
    ///
    /// Create new [Dcs] instance using a [WriteOnlyDataCommand]
    /// 
    pub fn write_only(di: DI) -> Self {
        Self { di }
    }

    ///
    /// Perform a software reset on the display
    /// 
    pub fn sw_reset(&mut self) -> Result<(), Error> {
        self.write_instruction(Instruction::SWRESET, &[])
    }

    ///
    /// Display on/off using [Instruction::DISPON] or [Instruction::DISPOFF]
    /// 
    pub fn display(&mut self, val: bool) -> Result<(), Error> {
        self.flip_command(val, Instruction::DISPON, Instruction::DISPOFF)
    }

    ///
    /// Normal mode using [Instruction::NORON] or [Instruction::PTLON]
    /// 
    pub fn mode_normal(&mut self, val: bool) -> Result<(), Error> {
        self.flip_command(val, Instruction::NORON, Instruction::PTLON)
    }

    ///
    /// Sleep mode using [Instruction::SLPIN] or [Instruction::SLPOUT]
    /// 
    pub fn mode_sleep(&mut self, val: bool) -> Result<(), Error> {
        self.flip_command(val, Instruction::SLPIN, Instruction::SLPOUT)
    }

    ///
    /// Color inversion using [Instruction::INVON] or [Instruction::INVOFF]
    /// 
    pub fn invert_colors(&mut self, val: bool) -> Result<(), Error> {
        self.flip_command(val, Instruction::INVON, Instruction::INVOFF)
    }

    ///
    /// Writes the specified DCS command "write only" using the provided display interface.
    /// 
    pub fn write_command(&mut self, command: impl DcsCommand) -> Result<(), Error> {
        self.write_instruction(command.instruction(), command.bytes())
    }

    ///
    /// Writes the specified DCS instruction and &[u8] parameters "write only"
    /// using the provided display interface. Use of `write_command` is preferred.
    /// 
    pub fn write_instruction(&mut self, instruction: Instruction, params: &[u8]) -> Result<(), Error> {
        self.di.send_commands(DataFormat::U8(&[instruction as u8]))?;

        if !params.is_empty() {
            self.di.send_data(DataFormat::U8(params))?;
            Ok(())
        } else {
            Ok(())
        }
    }

    // helper for "on/off" commands
    fn flip_command(&mut self, val: bool, cmd_on: Instruction, cmd_off: Instruction) -> Result<(), Error> {
        match val {
            true => self.write_instruction(cmd_on, &[]),
            false => self.write_instruction(cmd_off, &[]),
        }
    }
}
