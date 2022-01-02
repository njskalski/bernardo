use std::fmt::{Debug, Display, Formatter};

use crate::primitives::xy::XY;

//TODO find a shorter name

/*
Contracts:
x >= hint.x || x == None,
y >= hint.y || y == None,
None means "no limit"
 */
#[derive(Copy, Clone, Debug)]
pub struct SizeConstraint {
    x: Option<u16>,
    y: Option<u16>,

    // this corresponds to actual screen size. The idea is that layout is supposed to fill the
    // size constraint, but sometimes I want "widgets" that just fill the output without using
    // scroll - such widgets will use "hint" to get the size of display.
    hint: XY,
}

impl Display for SizeConstraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self.x {
            Some(x) => format!("{} (hint {})", x, self.hint.x),
            None => format!("unlimited (hint {})", self.hint.x),
        };

        let y = match self.y {
            Some(y) => format!("{} (hint {})", y, self.hint.y),
            None => format!("unlimited (hint {})", self.hint.y)
        };

        write!(f, "sc:[{}, {}]", x, y)
    }
}

impl SizeConstraint {
    pub fn new(x: Option<u16>, y: Option<u16>, hint: XY) -> Self {
        SizeConstraint {
            x,
            y,
            hint,
        }
    }

    pub fn simple(xy: XY) -> Self {
        SizeConstraint {
            x: Some(xy.x),
            y: Some(xy.y),
            hint: xy,
        }
    }

    pub fn x(&self) -> Option<u16> {
        self.x
    }

    pub fn y(&self) -> Option<u16> {
        self.y
    }

    pub fn hint(&self) -> XY {
        self.hint
    }

    pub fn bigger_equal_than(&self, xy: XY) -> bool {
        self.x.map(|x| x >= xy.x).unwrap_or(true) &&
            self.y.map(|y| y >= xy.y).unwrap_or(true)
    }

    pub fn strictly_bigger_than(&self, xy: XY) -> bool {
        self.x.map(|x| x > xy.x).unwrap_or(true) &&
            self.y.map(|y| y > xy.y).unwrap_or(true)
    }
}