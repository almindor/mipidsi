use embedded_hal::delay::DelayNs;

use crate::{
    dcs::{
        EnterNormalMode, ExitSleepMode, InterfaceExt, PixelFormat, SetAddressMode, SetDisplayOn,
        SetInvertMode, SetPixelFormat,
    },
    interface::AsyncInterface,
    options::ModelOptions,
};

/// Common init for all ILI934x controllers and color formats.
pub async fn init_common<DELAY, DI>(
    di: &mut DI,
    delay: &mut DELAY,
    options: &ModelOptions,
    pixel_format: PixelFormat,
) -> Result<SetAddressMode, DI::Error>
where
    DELAY: DelayNs,
    DI: AsyncInterface,
{
    let madctl = SetAddressMode::from(options);

    // 15.4:  It is necessary to wait 5msec after releasing RESX before sending commands.
    // 8.2.2: It will be necessary to wait 5msec before sending new command following software reset.
    delay.delay_us(5_000);

    di.write_command(madctl).await?;
    di.write_raw(0xB4, &[0x0]).await?;
    di.write_command(SetInvertMode::new(options.invert_colors))
        .await?;
    di.write_command(SetPixelFormat::new(pixel_format)).await?;

    di.write_command(EnterNormalMode).await?;

    // 8.2.12: It will be necessary to wait 120msec after sending Sleep In command (when in Sleep Out mode)
    //          before Sleep Out command can be sent.
    // The reset might have implicitly called the Sleep In command if the controller is reinitialized.
    delay.delay_us(120_000);

    di.write_command(ExitSleepMode).await?;

    // 8.2.12: It takes 120msec to become Sleep Out mode after SLPOUT command issued.
    // 13.2 Power ON Sequence: Delay should be 60ms + 80ms
    delay.delay_us(140_000);

    di.write_command(SetDisplayOn).await?;

    Ok(madctl)
}
