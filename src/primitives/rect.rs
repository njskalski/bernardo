use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

use crate::primitives::xy::XY;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub pos: XY,
    pub size: XY,
}

impl Rect {
    pub const fn new(pos: XY, size: XY) -> Self {
        Rect { pos, size }
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
}

pub struct CornersIterator {
    of: Rect,
    item: u8, // 0 is UpperLeft, 1 is UpperRight (clockwise). After 3 there is nothing.
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
        write!(f, "R[{},{}]", self.pos, self.size)
    }
}
