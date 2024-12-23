use std::cmp::min;

use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection::{Both, Vertical};
use crate::primitives::xy::XY;

#[derive(PartialEq, Eq)]
pub enum ScrollDirection {
    Horizontal,
    Vertical,
    Both,
}

impl ScrollDirection {
    pub fn free_x(&self) -> bool {
        *self == ScrollDirection::Horizontal || *self == ScrollDirection::Both
    }

    pub fn free_y(&self) -> bool {
        *self == Vertical || *self == Both
    }
}

pub struct Scroll {
    // TODO we DO need a vertical scroll of usize, 65k is not enough.
    pub offset: XY,
    pub direction: ScrollDirection,
}

impl Scroll {
    pub fn new(direction: ScrollDirection) -> Self {
        Scroll {
            offset: XY::ZERO,
            direction,
        }
    }

    pub fn follow_kite(&mut self, page_size: XY, max_output_size: XY, kite: XY) {
        debug_assert!(max_output_size > kite);
        debug_assert!(max_output_size >= page_size);

        let adjust_x = self.direction == ScrollDirection::Horizontal || self.direction == ScrollDirection::Both;
        let adjust_y = self.direction == ScrollDirection::Vertical || self.direction == ScrollDirection::Both;

        if adjust_x {
            if kite.x < self.offset.x {
                self.offset.x = kite.x;
            }

            if kite.x >= (self.offset.x + page_size.x) {
                self.offset.x = min(kite.x - page_size.x + 1, max_output_size.x - page_size.x);
            }
        }

        if adjust_y {
            if kite.y < self.offset.y {
                self.offset.y = kite.y;
            }

            if kite.y >= (self.offset.y + page_size.y) {
                self.offset.y = min(kite.y - page_size.y + 1, max_output_size.y - page_size.y);
            }
        }

        debug_assert!(Rect::new(self.offset, page_size).contains(kite));
    }
}
