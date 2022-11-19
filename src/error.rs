//! [Error] module for [super::Display]

use display_interface::DisplayError;

///
/// An error holding its source [embedded_hal::digital::OutputPin::Error]
/// or [display_interface::DisplayError]
///
#[derive(Debug)]
pub enum InitError<PE, DE> {
    DisplayError,
    Pin(PE),
    DelayError(DE),
}

///
/// Alias of [DisplayError] for out-of-init use cases
/// since the pin error is only possible during [super::Builder] use
///
pub type Error = DisplayError;

impl<PE, DE> From<DisplayError> for InitError<PE, DE> {
    fn from(_: DisplayError) -> Self {
        InitError::DisplayError
    }
}
