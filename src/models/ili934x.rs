use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565, Rgb666, RgbColor};
use embedded_hal::blocking::delay::DelayUs;

use crate::{
    dcs::{
        Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode, SetDisplayOn,
        SetInvertMode, SetPixelFormat, WriteMemoryStart,
    },
    Error, ModelOptions,
};

/// Common init for all ILI934x controllers and color formats.
pub fn init_common<DELAY, DI>(
    dcs: &mut Dcs<DI>,
    delay: &mut DELAY,
    options: &ModelOptions,
    pixel_format: PixelFormat,
) -> Result<SetAddressMode, Error>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    let madctl = SetAddressMode::from(options);

    // Datasheet info - 8.2.2. Software Reset (01h): It will be necessary to wait 5msec before sending new command following software reset. The display module loads all display
    // supplier factory default values to the registers during this 5msec
    delay.delay_us(5_000);

    dcs.write_command(ExitSleepMode)?; // turn off sleep

    // Datasheet info - 8.2.2. Software Reset (01h): If Software Reset is applied during Sleep Out mode, it will be
    // necessary to wait 120msec before sending Sleep out command. Software Reset Command cannot be sent during Sleep Out
    // sequence.
    delay.delay_us(120_000);

    dcs.write_command(madctl)?; // left -> right, bottom -> top RGB
    dcs.write_raw(0xB4, &[0x0])?; //Inversion Control [00]
    dcs.write_command(SetInvertMode(options.invert_colors))?; // set color inversion
    dcs.write_command(SetPixelFormat::new(pixel_format))?; // pixel format

    dcs.write_command(EnterNormalMode)?; // turn to normal mode

    dcs.write_command(SetDisplayOn)?; // turn on display

    Ok(madctl)
}

pub fn write_pixels_rgb565<DI, I>(dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    I: IntoIterator<Item = Rgb565>,
{
    dcs.write_command(WriteMemoryStart)?;
    let mut iter = colors.into_iter().map(|c| c.into_storage());

    let buf = DataFormat::U16BEIter(&mut iter);
    dcs.di.send_data(buf)
}

pub fn write_pixels_rgb666<DI, I>(dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    I: IntoIterator<Item = Rgb666>,
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
