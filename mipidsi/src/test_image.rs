use core::marker::PhantomData;

use embedded_graphics_core::{
    geometry::{AnchorPoint, AnchorX},
    prelude::*,
    primitives::Rectangle,
};

/// Test image.
///
/// The test image can be used to check if the display is working and to
/// identify the correct orientation and color settings.
pub struct TestImage<C: RgbColor> {
    color_type: PhantomData<C>,
}

impl<C: RgbColor> TestImage<C> {
    /// Creates a new test image.
    pub fn new() -> Self {
        Self {
            color_type: PhantomData,
        }
    }
}

const CORNER_SIZE: u32 = 10;
const CORNER_STROKE_WIDTH: u32 = 1;

impl<C: RgbColor> Drawable for TestImage<C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let bounding_box = target.bounding_box();
        target.fill_solid(&bounding_box, RgbColor::GREEN)?;
        Character::new(G, bounding_box.center()).draw(target)?;

        let rect = bounding_box.resized_width(bounding_box.size.width / 3, AnchorX::Left);
        target.fill_solid(&rect, RgbColor::RED)?;
        Character::new(R, rect.center()).draw(target)?;

        let rect = bounding_box.resized_width(bounding_box.size.width / 3, AnchorX::Right);
        target.fill_solid(&rect, RgbColor::BLUE)?;
        Character::new(B, rect.center()).draw(target)?;

        for anchor in [
            AnchorPoint::TopLeft,
            AnchorPoint::TopRight,
            AnchorPoint::BottomLeft,
            AnchorPoint::BottomRight,
        ] {
            target.fill_solid(
                &bounding_box.resized(Size::new(CORNER_SIZE, CORNER_STROKE_WIDTH), anchor),
                C::WHITE,
            )?;
            target.fill_solid(
                &bounding_box.resized(Size::new(CORNER_STROKE_WIDTH, CORNER_SIZE), anchor),
                C::WHITE,
            )?;
        }

        let top_left_marker = Rectangle::new(
            Point::new_equal((CORNER_STROKE_WIDTH * 3) as i32),
            Size::new_equal(CORNER_SIZE - 3 * CORNER_STROKE_WIDTH),
        );
        target.fill_solid(&top_left_marker, C::WHITE)?;

        Ok(())
    }
}

const R: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 1, 1, 1, 1, 0, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 1, 1, 1, 0, 0, 0, //
    0, 0, 1, 0, 1, 0, 0, 0, 0, //
    0, 0, 1, 0, 0, 1, 0, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
];

const G: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 1, 1, 1, 1, 0, 0, //
    0, 0, 1, 0, 0, 0, 0, 0, 0, //
    0, 0, 1, 0, 0, 0, 0, 0, 0, //
    0, 0, 1, 0, 1, 1, 1, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 0, 1, 1, 1, 1, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
];

const B: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 1, 1, 1, 1, 0, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 1, 1, 1, 0, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 1, 1, 1, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
];

struct Character<C> {
    data: &'static [u8],
    center: Point,
    color_type: PhantomData<C>,
}

impl<C> Character<C> {
    fn new(data: &'static [u8], center: Point) -> Self {
        Self {
            data,
            center,
            color_type: PhantomData,
        }
    }
}

impl<C: RgbColor> Drawable for Character<C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let rect = Rectangle::with_center(self.center, Size::new(9, 11));

        target.fill_contiguous(
            &rect,
            self.data.iter().map(|d| {
                if *d == 0 {
                    RgbColor::BLACK
                } else {
                    RgbColor::WHITE
                }
            }),
        )
    }
}
