//! [super::Display] builder module

use display_interface::WriteOnlyDataCommand;
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{error::InitError, models::Model, Display, ModelOptions};

pub struct DisplayBuilder<DI, MODEL>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    di: DI,
    model: MODEL,
    options: ModelOptions,
}

impl<DI, MODEL> DisplayBuilder<DI, MODEL>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
{
    ///
    /// Constructs a new builder from given [WriteOnlyDataCommand], [Model]
    /// and [ModelOptions]. For use by [Model] helpers, not public
    ///
    pub(crate) fn new(di: DI, model: MODEL, options: ModelOptions) -> Self {
        Self { di, model, options }
    }

    ///
    /// Consumes the builder to create a new [Display] with an optional reset [OutputPin].
    /// Blocks using the provided [DelayUs] `delay_source` to perform the display initialization.
    ///
    pub fn init<RST>(
        self,
        delay_source: &mut impl DelayUs<u32>,
        mut rst: Option<RST>,
    ) -> Result<Display<DI, MODEL>, InitError<RST::Error>>
    where
        RST: OutputPin,
    {
        let options = self.options;
        let madctl = options.madctl();

        let mut display = Display {
            di: self.di,
            model: self.model,
            options,
            madctl,
        };

        display.madctl = display
            .model
            .init(&mut display.di, delay_source, madctl, &mut rst)?;

        Ok(display)
    }
}
