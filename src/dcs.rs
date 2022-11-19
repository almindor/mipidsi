use display_interface::{DataFormat, WriteOnlyDataCommand};

use crate::{instruction::Instruction, Error, TearingEffect};

mod madctl;
pub use madctl::*;
mod colmod;
pub use colmod::*;
mod caset;
pub use caset::*;
mod raset;
pub use raset::*;
mod vscrdef;
pub use vscrdef::*;
mod vscad;
pub use vscad::*;

///
/// Provides a constructor for complex commands
/// e.g. `Madctl::new().with_bgr(true).bytes()`
///
pub trait DcsCommand {
    fn instruction(&self) -> Instruction;
    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, Error>;
}

///
/// Representation of the MIPI Display Command Set
/// Allows calling commands as methods with builder pattern
/// for the more complicated ones. Any command can be executed directly using the [Dcs::write_command] method.
/// Raw instructions can be sent using [Dcs::write_instruction].
/// Display interface can be accessed directly for data transfers using the `di` public field.
///
pub struct Dcs<DI> {
    ///
    /// Display interface instance
    ///
    pub di: DI,
}

impl<DI> Dcs<DI>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Create new [Dcs] instance using a [WriteOnlyDataCommand]
    ///
    pub fn write_only(di: DI) -> Self {
        Self { di }
    }

    ///
    /// Release the Display Interface back
    /// 
    pub fn release(self) -> DI {
        self.di
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
    pub fn display_on(&mut self, val: bool) -> Result<(), Error> {
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

    pub fn tearing_effect(&mut self, te: TearingEffect) -> Result<(), Error> {
        match te {
            TearingEffect::Off => self.write_instruction(Instruction::TEOFF, &[]),
            TearingEffect::Vertical => self.write_instruction(Instruction::TEON, &[0x0]),
            TearingEffect::HorizontalAndVertical => self.write_instruction(Instruction::TEON, &[0x1]),
        }
    }

    ///
    /// Call RAMWR preparing pixel writes
    ///
    pub fn prep_ram_write(&mut self) -> Result<(), Error> {
        self.write_instruction(Instruction::RAMWR, &[])
    }

    ///
    /// Writes the specified DCS command "write only" using the provided display interface.
    ///
    pub fn write_command(&mut self, command: &impl DcsCommand) -> Result<(), Error> {
        let mut param_bytes: [u8; 16] = [0; 16];
        let n = command.fill_params_buf(&mut param_bytes)?;
        self.write_instruction(command.instruction(), &param_bytes[..n])
    }

    ///
    /// Writes the specified DCS instruction and &[u8] parameters "write only"
    /// using the provided display interface. Use of `write_command` is preferred.
    ///
    pub fn write_instruction(
        &mut self,
        instruction: Instruction,
        param_bytes: &[u8],
    ) -> Result<(), Error> {
        self.di
            .send_commands(DataFormat::U8(&[instruction as u8]))?;

        if !param_bytes.is_empty() {
            self.di.send_data(DataFormat::U8(param_bytes))?; // TODO: empty guard?
        }
        Ok(())
    }

    // helper for "on/off" commands
    fn flip_command(
        &mut self,
        val: bool,
        cmd_on: Instruction,
        cmd_off: Instruction,
    ) -> Result<(), Error> {
        match val {
            true => self.write_instruction(cmd_on, &[]),
            false => self.write_instruction(cmd_off, &[]),
        }
    }
}
