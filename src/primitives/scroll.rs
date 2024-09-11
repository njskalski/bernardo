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

    pub fn follow_kite(&mut self, parent_output_size: XY, kite: XY) {
        let adjust_x = self.direction == ScrollDirection::Horizontal || self.direction == ScrollDirection::Both;
        let adjust_y = self.direction == ScrollDirection::Vertical || self.direction == ScrollDirection::Both;

        if adjust_x {
            if kite.x < self.offset.x {
                self.offset.x = kite.x;
            }

            if kite.x >= (self.offset.x + parent_output_size.x) {
                let diff = 1 + kite.x - (self.offset.x + parent_output_size.x);
                self.offset.x += diff;
            }
        }

        if adjust_y {
            if kite.y < self.offset.y {
                self.offset.y = kite.y;
            }

            if kite.y >= (self.offset.y + parent_output_size.y) {
                let diff = 1 + kite.y - (self.offset.y + parent_output_size.y);
                self.offset.y += diff;
            }
        }

        debug_assert!((self.offset + parent_output_size).x > kite.x);
        debug_assert!((self.offset + parent_output_size).y > kite.y);
    }
}
