use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

pub const ZERO: XY = XY::new(0, 0);

#[derive(Clone, Copy, Debug)]
pub struct XY {
    pub x: u16,
    pub y: u16,
}

impl XY {
    pub const fn new(x: u16, y: u16) -> Self {
        XY { x, y }
    }

    pub fn cut(&self, other: XY) -> XY {
        let minx = u16::min(self.x, other.x);
        let miny = u16::min(self.y, other.y);

        XY::new(minx, miny)
    }
}

impl From<(u16, u16)> for XY {
    fn from(pair: (u16, u16)) -> Self {
        XY {
            x: pair.0,
            y: pair.1,
        }
    }
}

impl From<(usize, usize)> for XY {
    fn from(pair: (usize, usize)) -> Self {
        debug_assert!(pair.0 < u16::MAX as usize);
        debug_assert!(pair.1 < u16::MAX as usize);

        XY {
            x: pair.0 as u16,
            y: pair.1 as u16,
        }
    }
}

impl From<(i32, i32)> for XY {
    fn from(pair: (i32, i32)) -> Self {
        debug_assert!(pair.0 < u16::MAX as i32);
        debug_assert!(pair.1 < u16::MAX as i32);

        XY {
            x: pair.0 as u16,
            y: pair.1 as u16,
        }
    }
}

impl Add for XY {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<(u16, u16)> for XY {
    type Output = Self;

    fn add(self, rhs: (u16, u16)) -> Self::Output {
        Self {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl PartialEq for XY {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn ne(&self, other: &Self) -> bool {
        self.x != other.x || self.y != other.y
    }
}

impl PartialOrd for XY {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x == other.x && self.y == other.y {
            return Some(Ordering::Equal);
        }

        if self.x > other.x && self.y > other.y {
            return Some(Ordering::Greater);
        }

        if self.x < other.x && self.y < other.y {
            return Some(Ordering::Greater);
        }

        None
    }

    fn lt(&self, other: &Self) -> bool {
        self.x < other.x && self.y < other.y
    }

    fn le(&self, other: &Self) -> bool {
        self.x <= other.x && self.y <= other.y
    }

    fn gt(&self, other: &Self) -> bool {
        self.x > other.x && self.y > other.y
    }

    fn ge(&self, other: &Self) -> bool {
        self.x >= other.x && self.y >= other.y
    }
}

impl fmt::Display for XY {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}
