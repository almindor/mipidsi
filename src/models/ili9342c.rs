use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{
    pixelcolor::{Rgb565, Rgb666},
    prelude::{IntoStorage, RgbColor},
};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    dcs::{Colmod, Dcs, Dispon, Madctl, Noron, Ramwr, Slpout, Swreset},
    error::InitError,
    instruction::Instruction,
    Builder, Error, ModelOptions,
};

use super::Model;

/// ILI9342C display with Reset pin
/// in Rgb565 color mode
/// Backlight pin is not controlled
pub struct ILI9342CRgb565;

/// ILI9342C display with Reset pin
/// in Rgb666 color mode
/// Backlight pin is not controlled
pub struct ILI9342CRgb666;

impl Model for ILI9342CRgb565 {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<Madctl, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(Swreset)?,
        }

        delay.delay_us(120_000);

        dcs.write_command(Colmod::new::<Self::ColorFormat>())?; // 16bit 65k colors

        Ok(init_common(dcs, delay, options)?)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(Ramwr)?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 240), (320, 240))
    }
}

impl Model for ILI9342CRgb666 {
    type ColorFormat = Rgb666;

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<Madctl, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(Swreset)?,
        }

        delay.delay_us(120_000);

        dcs.write_command(Colmod::new::<Self::ColorFormat>())?; // 18bit 262k colors

        Ok(init_common(dcs, delay, options)?)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(Ramwr)?;
        let mut iter = colors.into_iter().flat_map(|c| {
            let red = c.r() << 2;
            let green = c.g() << 2;
            let blue = c.b() << 2;
            [red, green, blue]
        });

        let buf = DataFormat::U8Iter(&mut iter);
        dcs.di.send_data(buf)
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((320, 240), (320, 240))
    }
}

// simplified constructor for Display

impl<DI> Builder<DI, ILI9342CRgb565>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9342C] as the [Model]
    /// with the default framebuffer size and display size of 320x240
    /// *WARNING* Rgb565 only works on non-SPI setups with the ILI9342C!
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9342c_rgb565(di: DI) -> Self {
        Self::with_model(di, ILI9342CRgb565)
    }
}

impl<DI> Builder<DI, ILI9342CRgb666>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ILI9342C] as the [Model]
    /// with the default framebuffer size and display size of 320x240
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    ///
    pub fn ili9342c_rgb666(di: DI) -> Self {
        Self::with_model(di, ILI9342CRgb666)
    }
}

// common init for all color format models
fn init_common<DELAY, DI>(
    dcs: &mut Dcs<DI>,
    delay: &mut DELAY,
    options: &ModelOptions,
) -> Result<Madctl, Error>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    let madctl = Madctl::from(options);

    dcs.write_command(Slpout)?; // turn off sleep
    dcs.write_command(madctl)?; // left -> right, bottom -> top RGB
    dcs.write_raw(Instruction::INVCO, &[0x0])?; //Inversion Control [00]
    dcs.write_command(options.invert_colors)?; // set color inversion

    dcs.write_command(Noron)?; // turn to normal mode
    dcs.write_command(Dispon)?; // turn on display

    // DISPON requires some time otherwise we risk SPI data issues
    delay.delay_us(120_000);

    Ok(madctl)
}
