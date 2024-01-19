//! Original code from: [this repo](https://github.com/lupyuen/piet-embedded/blob/master/piet-embedded-graphics/src/batch.rs)
//! Batch the pixels to be rendered into Pixel Rows and Pixel Blocks (contiguous Pixel Rows).
//! This enables the pixels to be rendered efficiently as Pixel Blocks, which may be transmitted in a single Non-Blocking SPI request.
use crate::{models::Model, Display, Error};
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics_core::prelude::*;
use embedded_hal::digital::OutputPin;
use mipidsi::batch::{to_blocks, to_rows, PixelBlock};

pub trait DrawBatch<DI, M, I>
where
    DI: AsyncWriteOnlyDataCommand,
    M: Model,
    I: IntoIterator<Item = Pixel<M::ColorFormat>>,
{
    async fn draw_batch(&mut self, item_pixels: I) -> Result<(), Error>;
}

impl<DI, M, RST, I> DrawBatch<DI, M, I> for Display<DI, M, RST>
where
    DI: AsyncWriteOnlyDataCommand,
    M: Model,
    I: IntoIterator<Item = Pixel<M::ColorFormat>>,
    RST: OutputPin,
{
    async fn draw_batch(&mut self, item_pixels: I) -> Result<(), Error> {
        //  Get the pixels for the item to be rendered.
        let pixels = item_pixels.into_iter();
        //  Batch the pixels into Pixel Rows.
        let rows = to_rows(pixels);
        //  Batch the Pixel Rows into Pixel Blocks.
        let blocks = to_blocks(rows);
        //  For each Pixel Block...
        for PixelBlock {
            x_left,
            x_right,
            y_top,
            y_bottom,
            colors,
            ..
        } in blocks
        {
            //  Render the Pixel Block.
            self.set_pixels(x_left, y_top, x_right, y_bottom, colors)
                .await?;

            //  Dump out the Pixel Blocks for the square in test_display()
            /* if x_left >= 60 && x_left <= 150 && x_right >= 60 && x_right <= 150 && y_top >= 60 && y_top <= 150 && y_bottom >= 60 && y_bottom <= 150 {
                console::print("pixel block ("); console::printint(x_left as i32); console::print(", "); console::printint(y_top as i32); ////
                console::print("), ("); console::printint(x_right as i32); console::print(", "); console::printint(y_bottom as i32); console::print(")\n"); ////
            } */
        }
        Ok(())
    }
}
