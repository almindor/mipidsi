//! Module for the MADCTL instruction constructors

use crate::{instruction::Instruction, ColorOrder, Orientation, RefreshOrder};

use super::DcsCommand;

pub struct Madctl(u8);

impl Madctl {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn color_order(mut self, color_order: ColorOrder) -> Self {
        match color_order {
            ColorOrder::Rgb => self.0 &= 0b1111_0111,
            ColorOrder::Bgr => self.0 |= 0b0000_1000,
        }

        self
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        let value = match orientation {
            Orientation::Portrait(false) => 0b0000_0000,
            Orientation::Portrait(true) => 0b0100_0000,
            Orientation::PortraitInverted(false) => 0b1100_0000,
            Orientation::PortraitInverted(true) => 0b1000_0000,
            Orientation::Landscape(false) => 0b0010_0000,
            Orientation::Landscape(true) => 0b0110_0000,
            Orientation::LandscapeInverted(false) => 0b1110_0000,
            Orientation::LandscapeInverted(true) => 0b1010_0000,
        };
        self.0 = (self.0 & 0b0001_1111) | value;

        self
    }

    pub fn refresh_order(mut self, refresh_order: RefreshOrder) -> Self {
        let value = match refresh_order {
            RefreshOrder::Normal => 0b0000_0000,
            RefreshOrder::RightToLeft => 0b0000_0100,
            RefreshOrder::BottomToTop => 0b0001_0000,
            RefreshOrder::Inverted => 0b0001_0100,
        };

        self.0 = (self.0 & 0b1110_1011) | value;

        self
    }
}

impl DcsCommand for Madctl {
    fn instruction(&self) -> Instruction {
        Instruction::MADCTL
    }

    fn bytes(&self) -> &[u8] {
        core::slice::from_ref(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn madctl_bit_operations() {
        let madctl = Madctl::new()
            .color_order(ColorOrder::Bgr)
            .refresh_order(RefreshOrder::Inverted)
            .orientation(Orientation::LandscapeInverted(true));
        assert_eq!(madctl.bytes(), &[0b1011_1100]);

        let madctl = madctl.orientation(Orientation::default());
        assert_eq!(madctl.bytes(), &[0b0001_1100]);

        let madctl = madctl.color_order(ColorOrder::Rgb);
        assert_eq!(madctl.bytes(), &[0b0001_0100]);

        let madctl = madctl.refresh_order(RefreshOrder::Normal);
        assert_eq!(madctl.bytes(), &[0b0000_0000]);
    }
}
