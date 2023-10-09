use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565, Rgb666, RgbColor};
use embedded_hal::blocking::delay::DelayUs;

use crate::{
    dcs::{
        EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode, SetDisplayOn, SetInvertMode,
        SetPixelFormat, WriteMemoryStart,
    },
    Display, Error,
};

use super::Controller;

/// Common init for all ILI934x controllers and color formats.
pub fn init_common<C, DI, RST, DELAY>(
    display: &mut Display<C, DI, RST>,
    delay: &mut DELAY,
    pixel_format: PixelFormat,
) -> Result<(), Error>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    let madctl = SetAddressMode::new(
        display.color_order,
        display.orientation,
        display.refresh_order,
    );

    // 15.4:  It is necessary to wait 5msec after releasing RESX before sending commands.
    // 8.2.2: It will be necessary to wait 5msec before sending new command following software reset.
    delay.delay_us(5_000);

    display.dcs.write_command(madctl)?;
    display.dcs.write_raw(0xB4, &[0x0])?;
    display
        .dcs
        .write_command(SetInvertMode(display.invert_colors))?;
    display
        .dcs
        .write_command(SetPixelFormat::new(pixel_format))?;

    display.dcs.write_command(EnterNormalMode)?;

    // 8.2.12: It will be necessary to wait 120msec after sending Sleep In command (when in Sleep Out mode)
    //          before Sleep Out command can be sent.
    // The reset might have implicitly called the Sleep In command if the controller is reinitialized.
    delay.delay_us(120_000);

    display.dcs.write_command(ExitSleepMode)?;

    // 8.2.12: It takes 120msec to become Sleep Out mode after SLPOUT command issued.
    // 13.2 Power ON Sequence: Delay should be 60ms + 80ms
    delay.delay_us(140_000);

    display.dcs.write_command(SetDisplayOn)?;

    Ok(())
}

pub fn write_pixels_rgb565<C: Controller, DI, RST, I>(
    display: &mut Display<C, DI, RST>,
    colors: I,
) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    I: IntoIterator<Item = Rgb565>,
{
    display.dcs.write_command(WriteMemoryStart)?;
    let mut iter = colors.into_iter().map(|c| c.into_storage());

    let buf = DataFormat::U16BEIter(&mut iter);
    display.dcs.di.send_data(buf)
}

pub fn write_pixels_rgb666<C: Controller, DI, RST, I>(
    display: &mut Display<C, DI, RST>,
    colors: I,
) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    I: IntoIterator<Item = Rgb666>,
{
    display.dcs.write_command(WriteMemoryStart)?;
    let mut iter = colors.into_iter().flat_map(|c| {
        let red = c.r() << 2;
        let green = c.g() << 2;
        let blue = c.b() << 2;
        [red, green, blue]
    });

    let buf = DataFormat::U8Iter(&mut iter);
    display.dcs.di.send_data(buf)
}
