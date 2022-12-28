use display_interface::{DataFormat, WriteOnlyDataCommand};

use crate::{instruction::Instruction, Error};

#[macro_use]
mod macros;

mod set_address_mode;
pub use set_address_mode::*;
mod set_pixel_format;
pub use set_pixel_format::*;
mod set_column_address;
pub use set_column_address::*;
mod set_page_address;
pub use set_page_address::*;
mod set_scroll_area;
pub use set_scroll_area::*;
mod set_scroll_start;
pub use set_scroll_start::*;
mod set_tearing_effect;
pub use set_tearing_effect::*;

///
/// Provides a constructor for DCS commands
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
    /// Writes the specified DCS command "write only" using the provided display interface.
    ///
    pub fn write_command(&mut self, command: impl DcsCommand) -> Result<(), Error> {
        let mut param_bytes: [u8; 16] = [0; 16];
        let n = command.fill_params_buf(&mut param_bytes)?;
        self.write_raw(command.instruction(), &param_bytes[..n])
    }

    ///
    /// Writes the specified DCS instruction and &[u8] parameters "write only"
    /// using the provided display interface. Use of `write_command` is preferred.
    ///
    pub fn write_raw(&mut self, instruction: Instruction, param_bytes: &[u8]) -> Result<(), Error> {
        self.di
            .send_commands(DataFormat::U8(&[instruction as u8]))?;

        if !param_bytes.is_empty() {
            self.di.send_data(DataFormat::U8(param_bytes))?; // TODO: empty guard?
        }
        Ok(())
    }
}

// DCS commands that don't use any parameters
dcs_basic_command!(SoftReset, Instruction::SWRESET);
dcs_basic_command!(EnterSleepMode, Instruction::SLPIN);
dcs_basic_command!(ExitSleepMode, Instruction::SLPOUT);
dcs_basic_command!(EnterPartialMode, Instruction::PTLON);
dcs_basic_command!(EnterNormalMode, Instruction::NORON);
dcs_basic_command!(SetDisplayOff, Instruction::DISPOFF);
dcs_basic_command!(SetDisplayOn, Instruction::DISPON);
dcs_basic_command!(ExitIdleMode, Instruction::IDLOFF);
dcs_basic_command!(EnterIdleMode, Instruction::IDLON);
dcs_basic_command!(ExitInvertMode, Instruction::INVOFF);
dcs_basic_command!(EnterInvertMode, Instruction::INVON);
dcs_basic_command!(WriteMemoryStart, Instruction::RAMWR);
