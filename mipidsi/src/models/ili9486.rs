use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{
    pixelcolor::{Rgb565, Rgb666},
    prelude::{IntoStorage, RgbColor},
};
use embedded_hal::delay::DelayNs;

use crate::{
    dcs::{
        BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat, WriteMemoryStart,
    },
    error::Error,
    options::ModelOptions,
};

use super::Model;

/// ILI9486 display in Rgb565 color mode.
pub struct ILI9486Rgb565;

/// ILI9486 display in Rgb666 color mode.
pub struct ILI9486Rgb666;

impl Model for ILI9486Rgb565 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 480);

    fn init<DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, Error>
    where
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
        delay.delay_us(120_000);

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        init_common(dcs, delay, options, pf)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)
    }

    fn repeat_pixel_to_buffer(color: Self::ColorFormat, buf: &mut [u8]) -> Result<usize, Error> {
        super::repeat_pixel_to_buffer_rgb565(color, buf)
    }
}

impl Model for ILI9486Rgb666 {
    type ColorFormat = Rgb666;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 480);

    fn init<DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, Error>
    where
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
        delay.delay_us(120_000);

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        init_common(dcs, delay, options, pf)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;
        let mut iter = colors.into_iter().flat_map(|c| {
            let red = c.r() << 2;
            let green = c.g() << 2;
            let blue = c.b() << 2;
            [red, green, blue]
        });

        let buf = DataFormat::U8Iter(&mut iter);
        dcs.di.send_data(buf)
    }

    fn repeat_pixel_to_buffer(color: Self::ColorFormat, buf: &mut [u8]) -> Result<usize, Error> {
        super::repeat_pixel_to_buffer_rgb666(color, buf)
    }
}

// common init for all color format models
fn init_common<DELAY, DI>(
    dcs: &mut Dcs<DI>,
    delay: &mut DELAY,
    options: &ModelOptions,
    pixel_format: PixelFormat,
) -> Result<SetAddressMode, Error>
where
    DELAY: DelayNs,
    DI: WriteOnlyDataCommand,
{
    let madctl = SetAddressMode::from(options);
    dcs.write_command(ExitSleepMode)?; // turn off sleep
    dcs.write_command(SetPixelFormat::new(pixel_format))?; // pixel format
    dcs.write_command(madctl)?; // left -> right, bottom -> top RGB
                                // dcs.write_command(Instruction::VCMOFSET, &[0x00, 0x48, 0x00, 0x48])?; //VCOM  Control 1 [00 40 00 40]
                                // dcs.write_command(Instruction::INVCO, &[0x0])?; //Inversion Control [00]
    dcs.write_command(SetInvertMode::new(options.invert_colors))?;

    // optional gamma setup
    // dcs.write_raw(Instruction::PGC, &[0x00, 0x2C, 0x2C, 0x0B, 0x0C, 0x04, 0x4C, 0x64, 0x36, 0x03, 0x0E, 0x01, 0x10, 0x01, 0x00])?; // Positive Gamma Control
    // dcs.write_raw(Instruction::NGC, &[0x0F, 0x37, 0x37, 0x0C, 0x0F, 0x05, 0x50, 0x32, 0x36, 0x04, 0x0B, 0x00, 0x19, 0x14, 0x0F])?; // Negative Gamma Control

    dcs.write_raw(0xB6, &[0b0000_0010, 0x02, 0x3B])?; // DFC
    dcs.write_command(EnterNormalMode)?; // turn to normal mode
    dcs.write_command(SetDisplayOn)?; // turn on display

    // DISPON requires some time otherwise we risk SPI data issues
    delay.delay_us(120_000);

    Ok(madctl)
}
