use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::{
    pixelcolor::{Rgb565, Rgb666},
    prelude::{IntoStorage, RgbColor},
};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{instruction::Instruction, Display, DisplayOptions, Error};

use super::{write_command, Model, ModelOptions};

/// ILI9342C display with Reset pin
/// in Rgb565 color mode
/// Backlight pin is not controlled
pub struct ILI9342CRgb565(ModelOptions);

/// ILI9342C display with Reset pin
/// in Rgb666 color mode
/// Backlight pin is not controlled
pub struct ILI9342CRgb666(ModelOptions);

impl Model for ILI9342CRgb565 {
    type ColorFormat = Rgb565;

    fn new(options: ModelOptions) -> Self {
        Self(options)
    }

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        rst: &mut Option<RST>,
        delay: &mut DELAY,
    ) -> Result<u8, Error<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }

        delay.delay_us(120_000);

        write_command(di, Instruction::COLMOD, &[0b0101_0101])?; // 16bit 65k colors

        init_common(di, delay, &self.0).map_err(|_| Error::DisplayError)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        di.send_data(buf)
    }

    // fn display_size(&self, orientation: Orientation) -> (u16, u16) {
    //     self.0.display_size(320, 240, orientation)
    // }

    fn options(&self) -> &ModelOptions {
        &self.0
    }
}

impl Model for ILI9342CRgb666 {
    type ColorFormat = Rgb666;

    fn new(options: ModelOptions) -> Self {
        Self(options)
    }

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        rst: &mut Option<RST>,
        delay: &mut DELAY,
    ) -> Result<u8, Error<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => write_command(di, Instruction::SWRESET, &[])?,
        }

        delay.delay_us(120_000);

        write_command(di, Instruction::COLMOD, &[0b0110_0110])?; // 18bit 262k colors

        init_common(di, delay, &self.0).map_err(|_| Error::DisplayError)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        write_command(di, Instruction::RAMWR, &[])?;
        let mut iter = colors.into_iter().flat_map(|c| {
            let red = c.r() << 2;
            let green = c.g() << 2;
            let blue = c.b() << 2;
            [red, green, blue]
        });

        let buf = DataFormat::U8Iter(&mut iter);
        di.send_data(buf)
    }

    // fn display_size(&self, orientation: Orientation) -> (u16, u16) {
    //     match orientation {
    //         Orientation::Portrait(_) | Orientation::PortraitInverted(_) => (320, 240),
    //         Orientation::Landscape(_) | Orientation::LandscapeInverted(_) => (240, 320),
    //     }
    // }

    fn options(&self) -> &ModelOptions {
        &self.0
    }
}

// simplified constructor for Display

impl<DI, RST> Display<DI, RST, ILI9342CRgb565>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Creates a new [Display] instance with [ILI9342C] as the [Model]
    /// *WARNING* Rgb565 only works on non-SPI setups with the ILI9342C!
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn ili9342c_rgb565(di: DI, rst: Option<RST>, options: DisplayOptions) -> Self {
        Self::with_model(
            di,
            rst,
            ILI9342CRgb565::new(ModelOptions::with_display_size(options, 320, 240)),
        )
    }
}

impl<DI, RST> Display<DI, RST, ILI9342CRgb666>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
{
    ///
    /// Creates a new [Display] instance with [ILI9342C] as the [Model]
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `rst` - display hard reset [OutputPin]
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn ili9342c_rgb666(di: DI, rst: Option<RST>, options: DisplayOptions) -> Self {
        Self::with_model(
            di,
            rst,
            ILI9342CRgb666::new(ModelOptions::with_display_size(options, 320, 240)),
        )
    }
}

// common init for all color format models
fn init_common<DELAY, DI>(
    di: &mut DI,
    delay: &mut DELAY,
    options: &ModelOptions,
) -> Result<u8, DisplayError>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    let madctl = options.madctl() ^ 0b0000_1000; // this model has flipped RGB/BGR bit;

    write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
    write_command(di, Instruction::MADCTL, &[madctl])?; // left -> right, bottom -> top RGB
    write_command(di, Instruction::INVCO, &[0x0])?; //Inversion Control [00]

    write_command(di, Instruction::NORON, &[])?; // turn to normal mode
    write_command(di, Instruction::DISPON, &[])?; // turn on display

    // DISPON requires some time otherwise we risk SPI data issues
    delay.delay_us(120_000);

    Ok(madctl)
}
