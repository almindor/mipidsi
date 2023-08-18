use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{delay::DelayUs, digital::OutputPin};

use crate::{
    dcs::{
        BitsPerPixel, ExitSleepMode, PixelFormat, SetAddressMode, SetDisplayOn, SetInvertMode,
        SetPixelFormat, SoftReset, WriteMemoryStart,
    },
    error::InitError,
    Builder, Error, ModelOptions,
};

use super::{Dcs, Model};

#[cfg(feature = "async")]
use crate::dcs::DcsAsync;
#[cfg(feature = "async")]
use crate::models::ModelAsync;
#[cfg(feature = "async")]
use display_interface::AsyncWriteOnlyDataCommand;
#[cfg(feature = "async")]
use embedded_hal_async::delay::DelayUs as DelayUsAsync;

/// GC9A01 display in Rgb565 color mode.
#[maybe_async_cfg::maybe(sync(keep_self), async(feature = "async"))]
pub struct GC9A01;

#[maybe_async_cfg::maybe(
    idents(
        Dcs(sync, async = "DcsAsync"),
        DelayUs(sync, async = "DelayUsAsync"),
        Model(sync, async = "ModelAsync"),
        WriteOnlyDataCommand(sync, async = "AsyncWriteOnlyDataCommand"),
    ),
    sync(keep_self),
    async(feature = "async")
)]
impl Model for GC9A01 {
    type ColorFormat = Rgb565;

    async fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayUs,
        DI: WriteOnlyDataCommand,
    {
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay).await?,
            None => dcs.write_command(SoftReset).await?,
        }
        delay.delay_us(200_000).await;

        dcs.write_raw(0xEF, &[]).await?; // inter register enable 2
        dcs.write_raw(0xEB, &[0x14]).await?;
        dcs.write_raw(0xFE, &[]).await?; // inter register enable 1
        dcs.write_raw(0xEF, &[]).await?; // inter register enable 2
        dcs.write_raw(0xEB, &[0x14]).await?;

        dcs.write_raw(0x84, &[0x40]).await?;
        dcs.write_raw(0x85, &[0xFF]).await?;
        dcs.write_raw(0x86, &[0xFF]).await?;
        dcs.write_raw(0x87, &[0xFF]).await?;
        dcs.write_raw(0x88, &[0x0A]).await?;
        dcs.write_raw(0x89, &[0x21]).await?;
        dcs.write_raw(0x8A, &[0x00]).await?;
        dcs.write_raw(0x8B, &[0x80]).await?;
        dcs.write_raw(0x8C, &[0x01]).await?;
        dcs.write_raw(0x8D, &[0x01]).await?;
        dcs.write_raw(0x8E, &[0xFF]).await?;
        dcs.write_raw(0x8F, &[0xFF]).await?;

        dcs.write_raw(0xB6, &[0x00, 0x20]).await?; // display function control

        dcs.write_command(madctl).await?; // set memory data access control, Top -> Bottom, RGB, Left -> Right

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf)).await?; // set interface pixel format, 16bit pixel into frame memory

        dcs.write_raw(0x90, &[0x08, 0x08, 0x08, 0x08]).await?;
        dcs.write_raw(0xBD, &[0x06]).await?;
        dcs.write_raw(0xBC, &[0x00]).await?;
        dcs.write_raw(0xFF, &[0x60, 0x01, 0x04]).await?;

        dcs.write_raw(0xC3, &[0x13]).await?; // power control 2
        dcs.write_raw(0xC4, &[0x13]).await?; // power control 3
        dcs.write_raw(0xC9, &[0x22]).await?; // power control 4

        dcs.write_raw(0xBE, &[0x11]).await?;
        dcs.write_raw(0xE1, &[0x10, 0x0E]).await?;
        dcs.write_raw(0xDF, &[0x20, 0x0c, 0x02]).await?;

        dcs.write_raw(0xF0, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A])
            .await?; // gamma 1
        dcs.write_raw(0xF1, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6f])
            .await?; // gamma 2
        dcs.write_raw(0xF2, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A])
            .await?; // gamma 3
        dcs.write_raw(0xF3, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6f])
            .await?; // gamma 4

        dcs.write_raw(0xED, &[0x18, 0x0B]).await?;
        dcs.write_raw(0xAE, &[0x77]).await?;
        dcs.write_raw(0xCD, &[0x63]).await?;

        dcs.write_raw(
            0x70,
            &[0x07, 0x07, 0x04, 0x0E, 0x0F, 0x09, 0x07, 0x08, 0x03],
        )
        .await?;

        dcs.write_raw(0xE8, &[0x34]).await?; // framerate

        dcs.write_raw(
            0x62,
            &[
                0x18, 0x0D, 0x71, 0xED, 0x70, 0x70, 0x18, 0x0F, 0x71, 0xEF, 0x70, 0x70,
            ],
        )
        .await?;
        dcs.write_raw(
            0x63,
            &[
                0x18, 0x11, 0x71, 0xF1, 0x70, 0x70, 0x18, 0x13, 0x71, 0xF3, 0x70, 0x70,
            ],
        )
        .await?;
        dcs.write_raw(0x64, &[0x28, 0x29, 0xF1, 0x01, 0xF1, 0x00, 0x07])
            .await?;
        dcs.write_raw(
            0x66,
            &[0x3C, 0x00, 0xCD, 0x67, 0x45, 0x45, 0x10, 0x00, 0x00, 0x00],
        )
        .await?;
        dcs.write_raw(
            0x67,
            &[0x00, 0x3C, 0x00, 0x00, 0x00, 0x01, 0x54, 0x10, 0x32, 0x98],
        )
        .await?;

        dcs.write_raw(0x74, &[0x10, 0x85, 0x80, 0x00, 0x00, 0x4E, 0x00])
            .await?;
        dcs.write_raw(0x98, &[0x3e, 0x07]).await?;

        dcs.write_command(SetInvertMode(options.invert_colors))
            .await?; // set color inversion

        dcs.write_command(ExitSleepMode).await?; // turn off sleep
        delay.delay_us(120_000).await;

        dcs.write_command(SetDisplayOn).await?; // turn on display

        Ok(madctl)
    }

    async fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart).await?;
        let mut iter = colors.into_iter().map(|c| c.into_storage());

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf).await?;
        Ok(())
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_sizes((240, 240), (240, 240))
    }
}

// simplified constructor on Display

impl<DI> Builder<DI, GC9A01>
where
    DI: WriteOnlyDataCommand,
{
    /// Creates a new display builder for GC9A01 displays in Rgb565 color mode.
    ///
    /// The default framebuffer size is 240x240 pixels and display size is 240x240 pixels.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](WriteOnlyDataCommand) for communicating with the display
    ///
    pub fn gc9a01(di: DI) -> Self {
        Self::with_model(di, GC9A01)
    }
}
