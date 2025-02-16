use core::marker::PhantomData;

use embedded_hal::delay::DelayNs;
use heapless::Vec;

use crate::{
    dcs::{DcsCommand, InterfaceExt},
    interface::{Interface, InterfaceKind},
    models::ModelInitError,
};

/// todo
pub trait InitEngine {
    const INTERFACE_KIND: InterfaceKind;
    type Error: core::fmt::Debug;
    /// Queue a raw command with optional parameters
    fn queue_command_raw(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error>;

    fn queue_command(&mut self, command: impl DcsCommand) -> Result<(), Self::Error>;

    fn queue_delay_us(&mut self, us: u32) -> Result<(), Self::Error>;

    fn pop_command(&mut self) -> Option<QueuedCommand>;
}

pub struct DirectEngine<'delay, DI, DELAY> {
    di: DI,
    delay: &'delay mut DELAY,
}

pub struct QueueEngine<DI> {
    queue: heapless::Deque<QueuedCommand, 24>,
    _di: PhantomData<DI>,
}

pub enum QueuedCommand {
    RawDcs(u8, Vec<u8, 16>),
    Delay(u32),
}

impl<DC> From<DC> for QueuedCommand
where
    DC: DcsCommand,
{
    fn from(value: DC) -> Self {
        let mut vec = Vec::<u8, 16>::new();
        value.fill_params_buf(vec.as_mut_slice());
        Self::RawDcs(value.instruction(), vec)
    }
}

impl<'delay, DI, DELAY> DirectEngine<'delay, DI, DELAY>
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

impl<DI> QueueEngine<DI>
where
    DI: Interface,
{
    pub fn new(_di: &DI) -> Self {
        Self {
            queue: heapless::Deque::new(),
            _di: PhantomData,
        }
    }
}

impl<DI, DELAY> InitEngine for DirectEngine<'_, DI, DELAY>
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

    fn pop_command(&mut self) -> Option<QueuedCommand> {
        None
    }
}

impl<DI> InitEngine for QueueEngine<DI>
where
    DI: Interface,
{
    const INTERFACE_KIND: InterfaceKind = DI::KIND;
    type Error = ModelInitError<DI::Error>;

    fn queue_command(&mut self, command: impl DcsCommand) -> Result<(), Self::Error> {
        self.queue
            .push_back(command.into())
            .map_err(|_| ModelInitError::InitEngineQueueFull)
    }

    fn queue_command_raw(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.queue
            .push_back(QueuedCommand::RawDcs(
                command,
                Vec::from_slice(args).unwrap(),
            ))
            .map_err(|_| ModelInitError::InitEngineQueueFull)
    }

    fn queue_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        self.queue
            .push_back(QueuedCommand::Delay(us))
            .map_err(|_| ModelInitError::InitEngineQueueFull)
    }

    fn pop_command(&mut self) -> Option<QueuedCommand> {
        self.queue.pop_front()
    }
}
