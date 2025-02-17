#[derive(Debug, Default)]
pub struct WindowExtents {
    pub x: (u16, u16),
    pub y: (u16, u16),
}

impl WindowExtents {
    pub fn default_max() -> Self {
        Self {
            x: (u16::MAX, 0),
            y: (u16::MAX, 0),
        }
    }

    pub fn apply_column_max(&mut self, sx: u16, ex: u16) {
        self.x.0 = core::cmp::min(self.x.0, sx);
        self.x.1 = core::cmp::max(self.x.1, ex);
    }

    pub fn apply_page_max(&mut self, sy: u16, ey: u16) {
        self.y.0 = core::cmp::min(self.y.0, sy);
        self.y.1 = core::cmp::max(self.y.1, ey);
    }
}

#[derive(Default, Debug)]
pub struct ExtentsRowIterator(usize);

impl ExtentsRowIterator {
    pub fn next(
        &mut self,
        extents: &WindowExtents,
        display_size: (u16, u16),
        pixel_bytes: usize,
    ) -> Option<(usize, usize)> {
        if self.0 + extents.y.0 as usize > extents.y.1 as usize {
            return None;
        }

        // these are all in pixel counts
        let start_x = extents.x.0 as usize;
        let start_y = extents.y.0 as usize + self.0;
        let size_x = (extents.x.1 - extents.x.0 + 1) as usize;

        // these are in byte counts
        let start_index = (start_x + display_size.0 as usize * start_y) * pixel_bytes;
        let end_index = start_index + (size_x * pixel_bytes);

        self.0 += 1;
        Some((start_index, end_index))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const PIXEL_BYTES: usize = 2;
    const DISPLAY_SIZE: (u16, u16) = (240, 130);

    #[test]
    fn test_zero_extents_row_iterator() {
        let extents = WindowExtents {
            x: (0, 0),
            y: (0, 0),
        };

        let mut iter = ExtentsRowIterator::default();

        if let Some((start_index, end_index)) = iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES) {
            assert_eq!(start_index, 0);
            assert_eq!(end_index, 2);
        } else {
            panic!("iterator did not return a value");
        }

        assert_eq!(iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES), None);
    }

    #[test]
    fn test_empty_extents_row_iterator() {
        let extents = WindowExtents {
            x: (44, 44),
            y: (33, 33),
        };

        let mut iter = ExtentsRowIterator::default();

        if let Some((start_index, end_index)) = iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES) {
            assert_eq!(
                start_index,
                (33 * DISPLAY_SIZE.0 as usize + 44) * PIXEL_BYTES
            );
            assert_eq!(end_index, start_index + PIXEL_BYTES);
        } else {
            panic!("iterator did not return a value");
        }

        assert_eq!(iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES), None);
    }

    #[test]
    fn test_rect_extents_row_iterator() {
        let extents = WindowExtents {
            x: (55, 57),
            y: (33, 33),
        };

        let mut iter = ExtentsRowIterator::default();

        if let Some((start_index, end_index)) = iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES) {
            assert_eq!(
                start_index,
                (33 * DISPLAY_SIZE.0 as usize + 55) * PIXEL_BYTES
            );
            assert_eq!(end_index, start_index + 3 * PIXEL_BYTES);
        } else {
            panic!("iterator did not return a value");
        }

        assert_eq!(iter.next(&extents, DISPLAY_SIZE, PIXEL_BYTES), None);
    }
}
