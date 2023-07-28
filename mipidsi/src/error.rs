//! [Error] module for [super::Display]

use display_interface::DisplayError;

/// Error returned by [`Builder::init`](crate::Builder).
#[derive(Debug)]
pub enum InitError<PE> {
    /// Error caused by the display interface.
    DisplayError,
    /// Error caused by the reset pin's [`OutputPin`](embedded_hal::digital::v2::OutputPin) implementation.
    Pin(PE),
}

///
/// Alias of [DisplayError] for out-of-init use cases
/// since the pin error is only possible during [super::Builder] use
///
pub type Error = DisplayError;

impl<PE> From<DisplayError> for InitError<PE> {
    fn from(_: DisplayError) -> Self {
        InitError::DisplayError
    }
}
