//! Error module for [super::Display]

/// Error returned by [`Builder::init`](crate::Builder).
#[derive(Debug)]
pub enum InitError<DI, P> {
    /// Error caused by the display interface.
    DisplayError(DI),
    /// Error caused by the reset pin's [`OutputPin`](embedded_hal::digital::OutputPin) implementation.
    Pin(P),
}
