use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565, Rgb666, RgbColor};
use embedded_hal::blocking::delay::DelayUs;

use crate::{instruction::Instruction, models::write_command, Error, ModelOptions};

/// Common init for all ILI934x controllers and color formats.
fn init_common<DELAY, DI>(
    di: &mut DI,
    delay: &mut DELAY,
    options: &ModelOptions,
) -> Result<u8, Error>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    let madctl = options.madctl();

    write_command(di, Instruction::SLPOUT, &[])?; // turn off sleep
    write_command(di, Instruction::MADCTL, &[madctl])?; // left -> right, bottom -> top RGB
    write_command(di, Instruction::INVCO, &[0x0])?; //Inversion Control [00]
    write_command(di, options.invert_command(), &[])?; // set color inversion

    write_command(di, Instruction::NORON, &[])?; // turn to normal mode
    write_command(di, Instruction::DISPON, &[])?; // turn on display

    // DISPON requires some time otherwise we risk SPI data issues
    delay.delay_us(120_000);

    Ok(madctl)
}

/// Common init for all ILI934x controllers with RGB565 color format.
pub fn init_rgb565<DELAY, DI>(
    di: &mut DI,
    delay: &mut DELAY,
    options: &ModelOptions,
) -> Result<u8, DisplayError>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    delay.delay_us(120_000);

    write_command(di, Instruction::COLMOD, &[0b0101_0101])?; // 16bit 65k colors

    Ok(init_common(di, delay, options)?)
}

/// Common init for all ILI934x controllers with RGB666 color format.
pub fn init_rgb666<DELAY, DI>(
    di: &mut DI,
    delay: &mut DELAY,
    options: &ModelOptions,
) -> Result<u8, DisplayError>
where
    DELAY: DelayUs<u32>,
    DI: WriteOnlyDataCommand,
{
    delay.delay_us(120_000);

    write_command(di, Instruction::COLMOD, &[0b0110_0110])?; // 18bit 262k colors

    Ok(init_common(di, delay, options)?)
}

pub fn write_pixels_rgb565<DI, I>(di: &mut DI, colors: I) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    I: IntoIterator<Item = Rgb565>,
{
    write_command(di, Instruction::RAMWR, &[])?;
    let mut iter = colors.into_iter().map(|c| c.into_storage());

    let buf = DataFormat::U16BEIter(&mut iter);
    di.send_data(buf)
}

pub fn write_pixels_rgb666<DI, I>(di: &mut DI, colors: I) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    I: IntoIterator<Item = Rgb666>,
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
