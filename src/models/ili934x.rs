use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565, Rgb666, RgbColor};
use embedded_hal::delay::DelayUs;

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
    DELAY: DelayUs,
    DI: WriteOnlyDataCommand,
{
    let madctl = SetAddressMode::from(options);

    // 15.4:  It is necessary to wait 5msec after releasing RESX before sending commands.
    // 8.2.2: It will be necessary to wait 5msec before sending new command following software reset.
    delay.delay_us(5_000);

    dcs.write_command(madctl)?;
    dcs.write_raw(0xB4, &[0x0])?;
    dcs.write_command(SetInvertMode(options.invert_colors))?;
    dcs.write_command(SetPixelFormat::new(pixel_format))?;

    dcs.write_command(EnterNormalMode)?;

    // 8.2.12: It will be necessary to wait 120msec after sending Sleep In command (when in Sleep Out mode)
    //          before Sleep Out command can be sent.
    // The reset might have implicitly called the Sleep In command if the controller is reinitialized.
    delay.delay_us(120_000);

    dcs.write_command(ExitSleepMode)?;

    // 8.2.12: It takes 120msec to become Sleep Out mode after SLPOUT command issued.
    // 13.2 Power ON Sequence: Delay should be 60ms + 80ms
    delay.delay_us(140_000);

    dcs.write_command(SetDisplayOn)?;

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
