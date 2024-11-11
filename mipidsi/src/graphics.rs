use embedded_graphics_core::{
    geometry::{OriginDimensions, Size},
    pixelcolor::RgbColor,
};
use embedded_hal::digital::OutputPin;

use crate::dcs::BitsPerPixel;
use crate::models::Model;
use crate::Display;
use display_interface::WriteOnlyDataCommand;

#[cfg(feature = "batch")]
mod batch;
#[cfg(not(feature = "batch"))]
mod direct;

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
