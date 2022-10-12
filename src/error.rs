//! [Error] module for [super::Display]

use display_interface::DisplayError;

///
/// An error holding its source (pins or SPI)
///
#[derive(Debug)]
pub enum Error<PE> {
    DisplayError,
    Pin(PE),
}

impl<PE> From<DisplayError> for Error<PE> {
    fn from(_: DisplayError) -> Self {
        Error::DisplayError
    }
}
