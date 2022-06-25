use core::convert::Infallible;

use embedded_hal::digital::{blocking::OutputPin, ErrorType, PinState};

/// The NoPin struct is here as a dummy implementation of the OutputPin trait
/// to handle the case when devices do not have a RST Pin and remove the need
/// for the user to use a real Pin as a fake one in this case.
#[derive(Default)]
pub struct NoPin;

impl ErrorType for NoPin {
    type Error = Infallible;
}

impl OutputPin for NoPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_state(&mut self, _state: PinState) -> Result<(), Self::Error> {
        Ok(())
    }
}
