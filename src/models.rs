//! Display models.

#![allow(async_fn_in_trait)]
use crate::{
    dcs::SetAddressMode, interface::AsyncInterface, options::ModelOptions, ConfigurationError,
};
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::delay::DelayNs;

mod ili9341;
mod ili934x;

pub use ili9341::*;

/// Display model.
pub trait Model {
    /// The color format.
    type ColorFormat: RgbColor;

    /// The framebuffer size in pixels.
    const FRAMEBUFFER_SIZE: (u16, u16);

    /// Initializes the display for this model with MADCTL from [crate::Display]
    /// and returns the value of MADCTL set by init
    async fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, ModelInitError<DI::Error>>
    where
        DELAY: DelayNs,
        DI: AsyncInterface;
}

/// Error returned by [`Model::init`].
///
/// This error type is used internally by implementations of the [`Model`]
/// trait.
pub enum ModelInitError<DiError> {
    /// Error caused by the display interface.
    AsyncInterface(DiError),

    /// Invalid configuration error.
    ///
    /// This error is returned when the configuration passed to the builder is
    /// invalid. For example, when the combination of bit depth and interface
    /// kind isn't supported by the selected model.
    InvalidConfiguration(ConfigurationError),
}

impl<DiError> From<DiError> for ModelInitError<DiError> {
    fn from(value: DiError) -> Self {
        Self::AsyncInterface(value)
    }
}
