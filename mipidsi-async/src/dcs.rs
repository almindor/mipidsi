//! MIPI DCS commands.

use display_interface::DataFormat;

use display_interface::AsyncWriteOnlyDataCommand;

use mipidsi::Error;

/// Common trait for DCS commands.
///
/// The methods in this traits are used to convert a DCS command into bytes.
pub trait DcsCommand {
    /// Returns the instruction code.
    fn instruction(&self) -> u8;

    /// Fills the given buffer with the command parameters.
    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, Error>;
}

/// Wrapper around [`WriteOnlyDataCommand`] with support for writing DCS commands.
///
/// Commands which are part of the manufacturer independent user command set can be sent to the
/// display by using the [`write_command`](Self::write_command) method with one of the command types
/// in this module.
///
/// All other commands, which do not have an associated type in this module, can be sent using
/// the [`write_raw`](Self::write_raw) method. The underlying display interface is also accessible
/// using the public [`di`](Self::di) field.
pub struct Dcs<DI> {
    /// Display interface instance.
    pub di: DI,
}

impl<DI> Dcs<DI>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Creates a new [Dcs] instance from a display interface.
    pub fn write_only(di: DI) -> Self {
        Self { di }
    }

    /// Releases the display interface.
    pub fn release(self) -> DI {
        self.di
    }

    /// Sends a DCS command to the display interface.
    pub async fn write_command(&mut self, command: impl DcsCommand) -> Result<(), Error> {
        let mut param_bytes: [u8; 16] = [0; 16];
        let n = command.fill_params_buf(&mut param_bytes)?;
        self.write_raw(command.instruction(), &param_bytes[..n])
            .await
    }

    /// Sends a raw command with the given `instruction` to the display interface.
    ///
    /// The `param_bytes` slice can contain the instruction parameters, which are sent as data after
    /// the instruction code was sent. If no parameters are required an empty slice can be passed to
    /// this method.
    ///
    /// This method is intended to be used for sending commands which are not part of the MIPI DCS
    /// user command set. Use [`write_command`](Self::write_command) for commands in the user
    /// command set.
    pub async fn write_raw(&mut self, instruction: u8, param_bytes: &[u8]) -> Result<(), Error> {
        self.di
            .send_commands(DataFormat::U8(&[instruction]))
            .await?;

        if !param_bytes.is_empty() {
            self.di.send_data(DataFormat::U8(param_bytes)).await?; // TODO: empty guard?
        }
        Ok(())
    }
}
