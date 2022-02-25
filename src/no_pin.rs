use embedded_hal::digital::v2::{OutputPin, PinState};

use core::convert::Infallible;

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
