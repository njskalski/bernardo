use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

use serde::{Deserialize, Serialize};

use crate::primitives::xy::XY;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub pos: XY,
    pub size: XY,
}

impl Rect {
    // TODO this is a degenerated case, I should use Options instead of magic symbols
    pub const ZERO: Rect = Rect::new(XY::ZERO, XY::ZERO);

    pub const fn new(pos: XY, size: XY) -> Self {
        Rect { pos, size }
    }

    pub const fn from_zero(size: XY) -> Self {
        Rect { pos: XY::ZERO, size }
    }

    pub fn xxyy(x1: u16, x2: u16, y1: u16, y2: u16) -> Self {
        let minx = std::cmp::min(x1, x2);
        let maxx = std::cmp::max(x1, x2);
        let miny = std::cmp::min(y1, y2);
        let maxy = std::cmp::max(y1, y2);

        Rect::new(XY::new(minx, miny), XY::new(maxx - minx, maxy - miny))
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

    pub fn intersect(&self, other: Rect) -> Option<Rect> {
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

    pub fn minus_shift(&self, vec: XY) -> Option<Rect> {
        let mut lower_right = self.lower_right();

        if lower_right.x >= vec.x {
            lower_right.x -= vec.x;
        } else {
            lower_right.x = 0;
        }

        if lower_right.y >= vec.y {
            lower_right.y -= vec.y;
        } else {
            lower_right.y = 0;
        }

        let mut new_upper_left = lower_right;
        if new_upper_left.x >= self.size.x {
            new_upper_left.x -= self.size.x;
        } else {
            new_upper_left.x = 0;
        }

        if new_upper_left.y >= self.size.y {
            new_upper_left.y -= self.size.y;
        } else {
            new_upper_left.y = 0;
        }

        if new_upper_left < lower_right {
            Some(Rect::new(new_upper_left, lower_right - new_upper_left))
        } else {
            None
        }
    }

    pub fn capped_at(&self, xy: XY) -> Option<Rect> {
        let upper_left = self.upper_left();

        if upper_left.x >= xy.x {
            return None;
        }
        if upper_left.y >= xy.y {
            return None;
        }

        let new_lower_right = self.lower_right().min_both_axis(xy);

        if self.upper_left() < new_lower_right {
            Some(Rect::new(self.pos, new_lower_right - self.pos))
        } else {
            None
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

    pub fn contains(&self, what: XY) -> bool {
        return self.upper_left() <= what && what < self.lower_right();
    }

    pub fn contains_rect(&self, what: Rect) -> bool {
        self.pos <= what.pos && self.lower_right() >= what.lower_right()
    }
}

pub struct CornersIterator {
    of: Rect,
    item: u8, // 0 is upper_left, 1 is upper_right (clockwise). After 3 there is nothing.
}

impl CornersIterator {
    pub fn new(of: Rect) -> Self {
        CornersIterator { of, item: 0 }
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
        Rect { pos: pair.0, size: pair.1 }
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

#[cfg(test)]
pub mod tests {
    use crate::primitives::rect::Rect;
    use crate::primitives::xy::XY;

    #[test]
    fn rect_contains_test() {
        let rect1 = Rect::new(XY::new(23, 16), XY::new(97, 5));

        assert_eq!(rect1.contains_rect(rect1), true);
    }

    #[test]
    fn rect_intersect_test() {
        assert_eq!(
            Rect::new(XY::new(48, 36), XY::new(25, 1)).intersect(Rect::new(XY::ZERO, XY::new(49, 1))),
            None
        );

        assert_eq!(
            Rect::new(XY::ZERO, XY::new(10, 10)).intersect(Rect::new(XY::ZERO, XY::new(4, 4))),
            Some(Rect::new(XY::ZERO, XY::new(4, 4)))
        );

        assert_eq!(
            Rect::new(XY::ZERO, XY::new(10, 10),).intersect(Rect::new(XY::new(10, 10), XY::new(4, 4))),
            None,
        );

        assert_eq!(
            Rect::new(XY::ZERO, XY::new(10, 10),).intersect(Rect::new(XY::new(9, 9), XY::new(4, 4))),
            Some(Rect::new(XY::new(9, 9), XY::new(1, 1))),
        );

        assert_eq!(
            Rect::new(XY::new(9, 9), XY::new(4, 4)).intersect(Rect::new(XY::ZERO, XY::new(10, 10))),
            Some(Rect::new(XY::new(9, 9), XY::new(1, 1))),
        );

        /*
          0 1 2 3 4 5 6 7 8 9 0 1 2 3
        0                     |
        1                     |
        2                     |
        3 .............       |
        4             |       |
        5             |       |
        6 ------------+-------.
        7             |
        8 ............|
        9
        */

        assert_eq!(
            Rect::new(XY::ZERO, XY::new(10, 6)).intersect(Rect::new(XY::new(0, 3), XY::new(6, 5))),
            Some(Rect::new(XY::new(0, 3), XY::new(6, 3))),
        );
    }

    #[test]
    fn minus_shift_test() {
        assert_eq!(Rect::new(XY::new(3, 3), XY::new(3, 3)).minus_shift(XY::new(7, 7)), None);

        assert_eq!(Rect::new(XY::new(3, 4), XY::new(3, 4)).minus_shift(XY::new(7, 7)), None);

        assert_eq!(
            Rect::new(XY::new(3, 4), XY::new(3, 4)).minus_shift(XY::new(5, 5)),
            Some(Rect::new(XY::new(0, 0), XY::new(1, 3)))
        );
    }
}
