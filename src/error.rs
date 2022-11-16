//! [Error] module for [super::Display]

use core::fmt::Display;

use display_interface::DisplayError;

///
/// An error holding its source [embedded_hal::digital::v2::OutputPin::Error]
/// or [display_interface::DisplayError]
///
#[derive(Debug)]
pub enum InitError<PE> {
    DisplayError,
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
