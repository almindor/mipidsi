use core::marker::PhantomData;

use embedded_hal::delay::DelayNs;

use crate::{
    dcs::DcsCommand,
    interface::{Interface, InterfaceAsync, InterfaceKind},
    models::ModelInitError,
};

/// todo
pub trait InitEngine {
    // required for interface checks in Model::init
    const INTERFACE_KIND: InterfaceKind;
    type Error: core::fmt::Debug;
    /// Queue a raw command with optional parameters
    fn queue_command_raw(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error>;

    fn queue_command(&mut self, command: impl DcsCommand) -> Result<(), Self::Error>;

    fn queue_delay_us(&mut self, us: u32) -> Result<(), Self::Error>;

    fn pop_command(&mut self) -> Option<QueuedInitCommand>;
}

pub struct InitEngineSync<'delay, DI, DELAY> {
    di: DI,
    delay: &'delay mut DELAY,
}

pub struct InitEngineAsync<DI> {
    queue: heapless::Deque<QueuedInitCommand, 24>,
    _di: PhantomData<DI>,
}

pub enum QueuedInitCommand {
    RawDcs(u8, [u8; 16]),
    Delay(u32),
}

impl<DC> From<DC> for QueuedInitCommand
where
    DC: DcsCommand,
{
    fn from(value: DC) -> Self {
        let mut buf: [u8; 16] = [0; 16];
        value.fill_params_buf(&mut buf);
        Self::RawDcs(value.instruction(), buf)
    }
}

impl<'delay, DI, DELAY> InitEngineSync<'delay, DI, DELAY>
where
    DI: Interface,
    DELAY: DelayNs,
{
    pub fn new(di: DI, delay: &'delay mut DELAY) -> Self {
        Self { di, delay }
    }

    pub fn release(self) -> DI {
        self.di
    }
}

impl<DI> InitEngineAsync<DI>
where
    DI: InterfaceAsync,
{
    pub fn new(_di: &DI) -> Self {
        Self {
            queue: heapless::Deque::new(),
            _di: PhantomData,
        }
    }
}

impl<DI, DELAY> InitEngine for InitEngineSync<'_, DI, DELAY>
where
    DI: Interface,
    DELAY: DelayNs,
{
    const INTERFACE_KIND: InterfaceKind = DI::KIND;
    type Error = DI::Error;

    fn queue_command(&mut self, command: impl DcsCommand) -> Result<(), Self::Error> {
        self.di.write_command(command)
    }

    fn queue_command_raw(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.di.send_command(command, args)
    }

    fn queue_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        self.delay.delay_us(us);

        Ok(())
    }

    fn pop_command(&mut self) -> Option<QueuedInitCommand> {
        None
    }
}

impl<DI> InitEngine for InitEngineAsync<DI>
where
    DI: InterfaceAsync,
{
    const INTERFACE_KIND: InterfaceKind = DI::KIND;
    type Error = ModelInitError<DI::Error>;

    fn queue_command(&mut self, command: impl DcsCommand) -> Result<(), Self::Error> {
        self.queue
            .push_back(command.into())
            .map_err(|_| ModelInitError::InitEngineQueueFull)
    }

    fn queue_command_raw(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        let mut arg_buf = [0u8; 16];
        arg_buf[..args.len()].copy_from_slice(args);

        self.queue
            .push_back(QueuedInitCommand::RawDcs(command, arg_buf))
            .map_err(|_| ModelInitError::InitEngineQueueFull)
    }

    fn queue_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        self.queue
            .push_back(QueuedInitCommand::Delay(us))
            .map_err(|_| ModelInitError::InitEngineQueueFull)
    }

    fn pop_command(&mut self) -> Option<QueuedInitCommand> {
        self.queue.pop_front()
    }
}
