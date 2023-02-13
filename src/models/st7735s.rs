use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

use crate::{
    dcs::{
        BitsPerPixel, ExitSleepMode, PixelFormat, SetAddressMode, SetDisplayOn, SetInvertMode,
        SetPixelFormat, SoftReset, WriteMemoryStart,
    },
    error::InitError,
    Builder, ColorInversion, Error, ModelOptions,
};

use super::{Dcs, Model};

/// ST7735s SPI display with Reset pin
/// Only SPI with DC pin interface is supported
pub struct ST7735s;

impl Model for ST7735s {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(SoftReset)?,
        }
        delay.delay_us(200_000);

        dcs.write_command(ExitSleepMode)?; // turn off sleep
        delay.delay_us(120_000);

        dcs.write_command(SetInvertMode(options.invert_colors))?; // set color inversion
        dcs.write_raw(0xB1, &[0x05, 0x3A, 0x3A])?; // set frame rate
        dcs.write_raw(0xB2, &[0x05, 0x3A, 0x3A])?; // set frame rate
        dcs.write_raw(0xB3, &[0x05, 0x3A, 0x3A, 0x05, 0x3A, 0x3A])?; // set frame rate
        dcs.write_raw(0xB4, &[0b0000_0011])?; // set inversion control
        dcs.write_raw(0xC0, &[0x62, 0x02, 0x04])?; // set power control 1
        dcs.write_raw(0xC1, &[0xC0])?; // set power control 2
        dcs.write_raw(0xC2, &[0x0D, 0x00])?; // set power control 3
        dcs.write_raw(0xC3, &[0x8D, 0x6A])?; // set power control 4
        dcs.write_raw(0xC4, &[0x8D, 0xEE])?; // set power control 5
        dcs.write_raw(0xC5, &[0x0E])?; // set VCOM control 1
        dcs.write_raw(
            0xE0,
            &[
                0x10, 0x0E, 0x02, 0x03, 0x0E, 0x07, 0x02, 0x07, 0x0A, 0x12, 0x27, 0x37, 0x00, 0x0D,
                0x0E, 0x10,
            ],
        )?; // set GAMMA +Polarity characteristics
        dcs.write_raw(
            0xE1,
            &[
                0x10, 0x0E, 0x03, 0x03, 0x0F, 0x06, 0x02, 0x08, 0x0A, 0x13, 0x26, 0x36, 0x00, 0x0D,
                0x0E, 0x10,
            ],
        )?; // set GAMMA -Polarity characteristics

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgbcolor::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf))?; // set interface pixel format, 16bit pixel into frame memory

        dcs.write_command(madctl)?; // set memory data access control, Top -> Bottom, RGB, Left -> Right
        dcs.write_command(SetDisplayOn)?; // turn on display

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((80, 160), (132, 162)).with_invert_colors(ColorInversion::Inverted)
    }
}

// simplified constructor on Display

impl<DI> Builder<DI, ST7735s>
where
    DI: WriteOnlyDataCommand,
{
    ///
    /// Creates a new [Display] instance with [ST7735s] as the [Model] with a
    /// hard reset Pin with the default framebuffer size of 132x162
    /// and display size of 80x160
    ///
    /// # Arguments
    ///
    /// * `di` - a [DisplayInterface](WriteOnlyDataCommand) for talking with the display
    /// * `options` - the [DisplayOptions] for this display/model
    ///
    pub fn st7735s(di: DI) -> Self {
        Self::with_model(di, ST7735s)
    }
}
