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

    pub fn is_deformed(&self) -> bool {
        self.size.x == 0 || self.size.y == 0
    }

    pub fn intersect(&self, other: Rect) -> Rect {
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
            Rect {
                pos: XY::new(xs[1].0, ys[1].0),
                size: XY::new(xs[2].0 - xs[1].0, ys[2].0 - ys[1].0),
            }
        } else {
            Rect {
                pos: XY::new(0 as u16, 0 as u16),
                size: XY::new(0 as u16, 0 as u16),
            }
        }
    }

    pub fn shift(&self, vec : XY) -> Rect {
        Rect {
            pos : self.pos + vec,
            size : self.size
        }
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
