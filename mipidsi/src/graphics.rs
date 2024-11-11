use embedded_graphics_core::{
    geometry::{Dimensions, OriginDimensions, Size},
    pixelcolor::RgbColor,
    pixelcolor::{raw::ToBytes, Rgb565, Rgb666},
};

use embedded_graphics_core::primitives::Rectangle;
use embedded_hal::digital::OutputPin;

use crate::models::Model;
use crate::Display;
use crate::{dcs::BitsPerPixel, error::Error};
use display_interface::WriteOnlyDataCommand;

#[cfg(feature = "batch")]
mod batch;
#[cfg(not(feature = "batch"))]
mod direct;

fn fill_contiguous<DI, M, RST, I>(
    display: &mut Display<DI, M, RST>,
    area: &Rectangle,
    colors: I,
) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    M: Model,
    RST: OutputPin,
    I: IntoIterator<Item = M::ColorFormat>,
{
    let intersection = area.intersection(&display.bounding_box());
    let Some(bottom_right) = intersection.bottom_right() else {
        // No intersection -> nothing to draw
        return Ok(());
    };

    // Unchecked casting to u16 cannot fail here because the values are
    // clamped to the display size which always fits in an u16.
    let sx = intersection.top_left.x as u16;
    let sy = intersection.top_left.y as u16;
    let ex = bottom_right.x as u16;
    let ey = bottom_right.y as u16;

    let count = intersection.size.width * intersection.size.height;

    let mut colors = colors.into_iter();

    if &intersection == area {
        // Draw the original iterator if no edge overlaps the framebuffer
        display.set_pixels(sx, sy, ex, ey, take_u32(colors, count))
    } else {
        // Skip pixels above and to the left of the intersection
        let mut initial_skip = 0;
        if intersection.top_left.y > area.top_left.y {
            initial_skip += intersection.top_left.y.abs_diff(area.top_left.y) * area.size.width;
        }
        if intersection.top_left.x > area.top_left.x {
            initial_skip += intersection.top_left.x.abs_diff(area.top_left.x);
        }
        if initial_skip > 0 {
            nth_u32(&mut colors, initial_skip - 1);
        }

        // Draw only the pixels which don't overlap the edges of the framebuffer
        let take_per_row = intersection.size.width;
        let skip_per_row = area.size.width - intersection.size.width;
        display.set_pixels(
            sx,
            sy,
            ex,
            ey,
            take_u32(TakeSkip::new(colors, take_per_row, skip_per_row), count),
        )
    }
}

// optimization helpers

/// Populate the provided buffer with [Rgb565] bytes. Used for optimized `fill_solid` paths.
pub fn repeat_pixel_to_buffer_rgb565(color: Rgb565, buf: &mut [u8]) -> Result<usize, Error> {
    let bytes = color.to_be_bytes();

    repeat_pixel_to_buffer_bytes(&bytes, buf)
}

/// Populate the provided buffer with [Rgb666] bytes. Used for optimized `fill_solid` paths.
pub fn repeat_pixel_to_buffer_rgb666(color: Rgb666, buf: &mut [u8]) -> Result<usize, Error> {
    let bytes = color.to_be_bytes();

    repeat_pixel_to_buffer_bytes(&bytes, buf)
}

fn repeat_pixel_to_buffer_bytes(bytes: &[u8], buf: &mut [u8]) -> Result<usize, Error> {
    let mut j = 0;
    for val in buf {
        *val = bytes[j];

        j += 1;
        if j >= bytes.len() {
            j = 0;
        }
    }

    Ok(bytes.len())
}

// used for fill_solid calls
struct IntersectionInfo {
    count: u32, // pixel count
    sx: u16,
    sy: u16,
    ex: u16,
    ey: u16,
}

// calculate pixel area for fill_solid calls
fn calculate_intersection(
    area: &Rectangle,
    bounding_box: &Rectangle,
) -> Result<Option<IntersectionInfo>, Error> {
    let area = area.intersection(bounding_box);
    let Some(bottom_right) = area.bottom_right() else {
        // No intersection -> nothing to draw
        return Ok(None);
    };

    let count = area.size.width * area.size.height;

    let sx = area.top_left.x as u16;
    let sy = area.top_left.y as u16;
    let ex = bottom_right.x as u16;
    let ey = bottom_right.y as u16;

    Ok(Some(IntersectionInfo {
        count,
        sx,
        sy,
        ex,
        ey,
    }))
}

impl<DI, MODEL, RST> OriginDimensions for Display<DI, MODEL, RST>
where
    DI: WriteOnlyDataCommand,
    MODEL: Model,
    RST: OutputPin,
{
    fn size(&self) -> Size {
        let ds = self.options.display_size();
        let (width, height) = (u32::from(ds.0), u32::from(ds.1));
        Size::new(width, height)
    }
}

impl BitsPerPixel {
    /// Returns the bits per pixel for a embedded-graphics [`RgbColor`].
    pub const fn from_rgb_color<C: RgbColor>() -> Self {
        let bpp = C::MAX_R.trailing_ones() + C::MAX_G.trailing_ones() + C::MAX_B.trailing_ones();

        match bpp {
            3 => Self::Three,
            8 => Self::Eight,
            12 => Self::Twelve,
            16 => Self::Sixteen,
            18 => Self::Eighteen,
            24 => Self::TwentyFour,
            _ => panic!("invalid RgbColor bits per pixel"),
        }
    }
}

/// An iterator that alternately takes and skips elements of another iterator.
struct TakeSkip<I> {
    iter: I,
    take: u32,
    take_remaining: u32,
    skip: u32,
}

impl<I> TakeSkip<I> {
    pub fn new(iter: I, take: u32, skip: u32) -> Self {
        Self {
            iter,
            take,
            take_remaining: take,
            skip,
        }
    }
}

impl<I: Iterator> Iterator for TakeSkip<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.take_remaining > 0 {
            self.take_remaining -= 1;
            self.iter.next()
        } else if self.take > 0 {
            self.take_remaining = self.take - 1;
            nth_u32(&mut self.iter, self.skip)
        } else {
            None
        }
    }
}

#[cfg(not(target_pointer_width = "16"))]
fn take_u32<I: Iterator>(iter: I, max_count: u32) -> impl Iterator<Item = I::Item> {
    iter.take(max_count.try_into().unwrap())
}

#[cfg(target_pointer_width = "16")]
fn take_u32<I: Iterator>(iter: I, max_count: u32) -> impl Iterator<Item = I::Item> {
    let mut count = 0;
    iter.take_while(move |_| {
        count += 1;
        count <= max_count
    })
}

#[cfg(not(target_pointer_width = "16"))]
fn nth_u32<I: Iterator>(mut iter: I, n: u32) -> Option<I::Item> {
    iter.nth(n.try_into().unwrap())
}

#[cfg(target_pointer_width = "16")]
fn nth_u32<I: Iterator>(mut iter: I, n: u32) -> Option<I::Item> {
    for _ in 0..n {
        iter.next();
    }
    iter.next()
}

#[cfg(test)]
mod test {
    use crate::dcs::BitsPerPixel;
    use embedded_graphics_core::pixelcolor::*;

    use super::TakeSkip;

    #[test]
    fn bpp_from_rgb_color_works() {
        assert_eq!(
            BitsPerPixel::from_rgb_color::<Rgb565>(),
            BitsPerPixel::Sixteen
        );
        assert_eq!(
            BitsPerPixel::from_rgb_color::<Rgb666>(),
            BitsPerPixel::Eighteen
        );
        assert_eq!(
            BitsPerPixel::from_rgb_color::<Rgb888>(),
            BitsPerPixel::TwentyFour
        );
    }

    #[test]
    #[should_panic]
    fn bpp_from_rgb_color_invalid_panics() {
        BitsPerPixel::from_rgb_color::<Rgb555>();
    }

    #[test]
    fn take_skip_iter() {
        let mut iter = TakeSkip::new(0..11, 3, 2);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        // Skip 3 and 4
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), Some(7));
        // Skip 8 and 9
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn take_skip_with_take_equals_zero() {
        // take == 0 should not cause an integer overflow or infinite loop and
        // just return None
        let mut iter = TakeSkip::new(0..11, 0, 2);
        assert_eq!(iter.next(), None);
    }
}
