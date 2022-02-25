use embedded_hal::digital::v2::{OutputPin, PinState};

use core::convert::Infallible;

/// The NoPin struct is here as a dummy implementation of the OutputPin trait
/// to handle the case when devices do not have a RST Pin and remove the need
/// for the user to use a real Pin as a fake one in this case.
#[derive(Default)]
pub struct NoPin;

impl OutputPin for NoPin {
    type Error = Infallible;

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
