use crate::primitives::scroll::ScrollDirection::{Both, Vertical};
use crate::primitives::xy::{XY, ZERO};

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
    pub offset: XY,
    pub direction: ScrollDirection,
}

impl Scroll {
    pub fn new(direction: ScrollDirection) -> Self {
        Scroll {
            offset: ZERO,
            direction,
        }
    }

    pub fn follow_anchor(&mut self, output_size: XY, anchor: XY) {
        let adjust_x = self.direction == ScrollDirection::Horizontal || self.direction == ScrollDirection::Both;
        let adjust_y = self.direction == ScrollDirection::Vertical || self.direction == ScrollDirection::Both;

        if adjust_x {
            if anchor.x < self.offset.x {
                self.offset.x = anchor.x;
            }

            if anchor.x >= (self.offset.x + output_size.x) {
                let diff = 1 + anchor.x - (self.offset.x + output_size.x);
                self.offset.x += diff;
            }
        }

        if adjust_y {
            if anchor.y < self.offset.y {
                self.offset.y = anchor.y;
            }

            if anchor.y >= (self.offset.y + output_size.y) {
                let diff = 1 + anchor.y - (self.offset.y + output_size.y);
                self.offset.y += diff;
            }
        }
    }
}