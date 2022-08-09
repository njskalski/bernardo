use std::cmp::{max, min};
use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

use crate::primitives::xy;
use crate::primitives::xy::XY;

pub const ZERO_RECT: Rect = Rect::new(xy::ZERO, xy::ZERO);

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub pos: XY,
    pub size: XY,
}

impl Rect {
    pub const fn new(pos: XY, size: XY) -> Self {
        Rect { pos, size }
    }

    pub fn xxyy(x1: u16, x2: u16, y1: u16, y2: u16) -> Self {
        let minx = std::cmp::min(x1, x2);
        let maxx = std::cmp::max(x1, x2);
        let miny = std::cmp::min(y1, y2);
        let maxy = std::cmp::max(y1, y2);

        Rect::new(
            XY::new(minx, miny),
            XY::new(maxx - minx, maxy - miny),
        )
    }

    pub fn lower_right(&self) -> XY {
        self.pos + self.size
    }
    pub fn upper_left(&self) -> XY {
        self.pos
    }

    pub fn min_x(&self) -> u16 {
        self.pos.x
    }
    pub fn max_x(&self) -> u16 {
        self.pos.x + self.size.x
    }
    pub fn min_y(&self) -> u16 {
        self.pos.y
    }
    pub fn max_y(&self) -> u16 {
        self.pos.y + self.size.y
    }

    pub fn max_xy(&self) -> XY {
        XY::new(self.pos.x + self.size.x, self.pos.y + self.size.y)
    }

    pub fn is_deformed(&self) -> bool {
        self.size.x == 0 || self.size.y == 0
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        //TODO write tests

        /*
        idea: I sort beginnings and ends by their position.
            I require two beginnings before two endings on BOTH axis to have any
            intersection. Otherwise I return deformed rect.
         */

        // true = beginning, false = end
        // why? So in case there's a draw on coordinate, I still DO NOT construct
        // a rect. Since upper limits are EXCLUSIVE, such rect would be empty anyway.
        let mut xs: Vec<(u16, bool)> = vec![
            (self.upper_left().x, true),
            (self.lower_right().x, false),
            (other.upper_left().x, true),
            (other.lower_right().x, false),
        ];
        let mut ys: Vec<(u16, bool)> = vec![
            (self.upper_left().y, true),
            (self.lower_right().y, false),
            (other.upper_left().y, true),
            (other.lower_right().y, false),
        ];

        xs.sort();
        ys.sort();

        if xs[0].1 == true && xs[1].1 == true && ys[0].1 == true && ys[1].1 == true {
            Some(Rect {
                pos: XY::new(xs[1].0, ys[1].0),
                size: XY::new(xs[2].0 - xs[1].0, ys[2].0 - ys[1].0),
            })
        } else {
            None
        }
    }

    pub fn shifted(&self, vec: XY) -> Rect {
        Rect {
            pos: self.pos + vec,
            size: self.size,
        }
    }

    pub fn corners(&self) -> CornersIterator {
        CornersIterator::new(*self)
    }

    // This will expand the Rect, that it contains xy.
    // Since lower-right is exclusive invariant holds: self.lower_right() > xy after this operation.
    pub fn expand_to_contain(&mut self, xy: XY) {
        if self.pos.x > xy.x {
            self.pos.x = xy.x;
        }
        if self.pos.y > xy.y {
            self.pos.y = xy.y;
        }

        // at this point we know, that self.pos.x <= xy.x and self.pos.y <= xy.y . We will use that
        // to avoid underflow errors.
        if self.lower_right().x <= xy.x && xy.x < u16::MAX {
            //even if self.pos.x == 0, xy.x < u16::MAX, so I can add safely
            self.size.x = xy.x - self.pos.x + 1;
        }

        if self.lower_right().y <= xy.y && xy.y < u16::MAX {
            //even if self.pos.x == 0, xy.x < u16::MAX, so I can add safely
            self.size.y = xy.y - self.pos.y + 1;
        }

        debug_assert!(self.lower_right().x > xy.x || xy.x == u16::MAX);
        debug_assert!(self.lower_right().y > xy.y || xy.y == u16::MAX);
    }
}

pub struct CornersIterator {
    of: Rect,
    item: u8, // 0 is upper_left, 1 is upper_right (clockwise). After 3 there is nothing.
}

impl CornersIterator {
    pub fn new(of: Rect) -> Self {
        CornersIterator {
            of,
            item: 0,
        }
    }
}

impl Iterator for CornersIterator {
    type Item = XY;

    fn next(&mut self) -> Option<Self::Item> {
        // Upper Left
        if self.item == 0 {
            self.item += 1;
            return Some(self.of.pos);
        };

        // Upper Right
        if self.item == 1 {
            self.item += 1;
            return Some(XY::new(self.of.pos.x + self.of.size.y - 1, self.of.pos.y));
        };

        // Bottom Right
        if self.item == 2 {
            self.item += 1;
            return Some(XY::new(self.of.pos.x + self.of.size.y - 1, self.of.pos.y + self.of.size.y - 1));
        };

        // Bottom Left
        if self.item == 3 {
            self.item += 1;
            return Some(XY::new(self.of.pos.x, self.of.pos.y + self.of.size.y - 1));
        };

        None
    }
}


impl From<(XY, XY)> for Rect {
    fn from(pair: (XY, XY)) -> Self {
        Rect {
            pos: pair.0,
            size: pair.1,
        }
    }
}

impl Add<XY> for Rect {
    type Output = Self;

    fn add(self, other: XY) -> Rect {
        Rect {
            pos: self.pos + other,
            size: self.size,
        }
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "r[{},{} ({})]", self.pos, self.size, self.lower_right())
    }
}
