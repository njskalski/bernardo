use std::cmp::{min, Ordering};
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use log::error;
use serde::{Deserialize, Serialize};

use crate::primitives::size_constraint::SizeConstraint;

#[derive(Clone, Copy, Debug, Hash, Eq, Serialize, Deserialize, Ord)]
pub struct XY {
    pub x: u16,
    pub y: u16,
}

impl XY {
    pub const ZERO: XY = XY::new(0, 0);
    pub const ONE: XY = XY::new(1, 1);

    pub const fn new(x: u16, y: u16) -> Self {
        XY { x, y }
    }

    pub fn cut(&self, sc: SizeConstraint) -> XY {
        let mut res = *self;

        match sc.x() {
            Some(x) => if x < res.x { res.x = x; },
            _ => {}
        }

        match sc.y() {
            Some(y) => if y < res.y { res.y = y; },
            _ => {}
        }

        res
    }

    pub fn maybe_minus(&self, other: XY) -> Option<XY> {
        if self.x >= other.x && self.y >= other.y {
            Some(XY::new(self.x - other.x, self.y - other.y))
        } else {
            None
        }
    }

    pub fn neighbours(&self) -> NeighboursIterator {
        NeighboursIterator::new(*self)
    }

    pub fn max_both_axis(&self, other: XY) -> XY {
        XY::new(
            u16::max(self.x, other.x),
            u16::max(self.y, other.y),
        )
    }

    pub fn min_both_axis(&self, other: XY) -> XY {
        XY::new(
            u16::min(self.x, other.x),
            u16::min(self.y, other.y),
        )
    }
}

impl Div<u16> for XY {
    type Output = XY;

    fn div(self, rhs: u16) -> Self::Output {
        XY::new(
            self.x / rhs,
            self.y / rhs,
        )
    }
}

impl Mul<usize> for XY {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self {
        let x = rhs * self.x as usize;
        let y = rhs * self.y as usize;
        if x > u16::MAX as usize || y > u16::MAX as usize {
            error!("mult would exceed u16 limit, using u16::MAX as fallback: {} * {}", self, rhs);
        }

        let x = min(x, u16::MAX as usize) as u16;
        let y = min(y, u16::MAX as usize) as u16;

        XY::new(x, y)
    }
}


pub struct NeighboursIterator {
    of: XY,
    item: u8, // 0 is North, 1 is Right (clockwise). After 3 there is nothing.
}

impl NeighboursIterator {
    pub fn new(of: XY) -> Self {
        NeighboursIterator {
            of,
            item: 0,
        }
    }
}

impl Iterator for NeighboursIterator {
    type Item = XY;

    fn next(&mut self) -> Option<Self::Item> {
        // neighbour above.
        if self.item == 0 {
            if self.of.y > 0 {
                self.item += 1;
                return Some(XY::new(self.of.x, self.of.y - 1));
            } else {
                self.item += 1;
            }
        };

        //neighbour on the right.
        if self.item == 1 {
            self.item += 1;
            return Some(XY::new(self.of.x + 1, self.of.y));
        };

        //neighbour below.
        if self.item == 2 {
            self.item += 1;
            return Some(XY::new(self.of.x + 1, self.of.y));
        };

        //neighbour on the left.
        if self.item == 3 {
            if self.of.x > 0 {
                self.item += 1;
                return Some(XY::new(self.of.x - 1, self.of.y));
            } else {
                self.item += 1;
            }
        };

        None
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

impl Sub for XY {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        if other.x > self.x || other.y > self.y {
            error!("failed substracting XY {} - {}, using fallback 0 for negative value", self, other);
        }

        XY::new(
            if self.x > other.x {
                self.x - other.x
            } else { 0 },
            if self.y > other.y {
                self.y - other.y
            } else { 0 },
        )
    }
}

impl SubAssign for XY {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl AddAssign for XY {
    fn add_assign(&mut self, rhs: XY) {
        self.x += rhs.x;
        self.y += rhs.y;
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
