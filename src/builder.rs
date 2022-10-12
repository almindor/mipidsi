//! [super::Display] builder module

use display_interface::WriteOnlyDataCommand;
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{error::InitError, models::Model, Display};

pub struct DisplayBuilder<DI, MODEL>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    di: DI,
    model: MODEL,
}

impl<DI, MODEL> DisplayBuilder<DI, MODEL>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    ///
    /// Constructs a new builder from given [WriteOnlyDataCommand] and [Model]
    ///
    pub fn new(di: DI, model: MODEL) -> Self {
        Self { di, model }
    }

    ///
    /// Consumes the builder to create a new [Display] with an optional reset [OutputPin].
    /// Blocks using the provided [DelayUs] `delay_source` to perform the display initialization.
    ///
    pub fn init<RST>(
        self,
        mut rst: Option<RST>,
        delay_source: &mut impl DelayUs<u32>,
    ) -> Result<Display<DI, MODEL>, InitError<RST::Error>>
    where
        RST: OutputPin,
    {
        let orientation = self.model.options().orientation();

        let mut display = Display {
            di: self.di,
            model: self.model,
            orientation,
            madctl: 0,
        };

        display.madctl = display
            .model
            .init(&mut display.di, &mut rst, delay_source)?;

        Ok(display)
    }
}
