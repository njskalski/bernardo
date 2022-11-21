use std::cmp::min;
use std::fmt::{Debug, Display, Formatter};

use log::{debug, error, warn};

use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

//TODO find a shorter name

/*
Contracts:
x >= hint.lower_right.x || x == None,
y >= hint.lower_right.y || y == None,
None means "no limit"
 */

pub enum AlignEnum {
    // uses as little space as possible
    Constrained,
    // fills entire dimension
    Greedy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SizeConstraint {
    // "infinite" dimension means we have a scroll, and widget is supposed to tell *how much it can
    // fill with meaningful output given infinite space*.
    x: Option<u16>,
    y: Option<u16>,

    // this corresponds to actual screen pos and size (visible part).
    visible: Rect,
}

impl Display for SizeConstraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self.x {
            Some(x) => format!("{} (hint {})", x, self.visible.size.x),
            None => format!("unlimited (hint {})", self.visible.size.x),
        };

        let y = match self.y {
            Some(y) => format!("{} (hint {})", y, self.visible.size.y),
            None => format!("unlimited (hint {})", self.visible.size.y)
        };

        write!(f, "sc:[{}, {}][off {}]", x, y, self.visible.pos)
    }
}

impl SizeConstraint {
    pub fn new(x: Option<u16>, y: Option<u16>, rect: Rect) -> Self {
        SizeConstraint {
            x,
            y,
            visible: rect,
        }
    }

    pub fn simple(xy: XY) -> Self {
        SizeConstraint {
            x: Some(xy.x),
            y: Some(xy.y),
            visible: Rect::new(XY::ZERO, xy),
        }
    }

    pub fn x(&self) -> Option<u16> {
        self.x
    }

    pub fn y(&self) -> Option<u16> {
        self.y
    }

    // This corresponds to VISIBLE PART of output. It is used for two things:
    // - drawing optimisation
    // - layouting views that want to "fill" the visible part.
    // - layouting anything that wants to be "centered" while x axis is unlimited.
    pub fn visible_hint(&self) -> &Rect {
        &self.visible
    }

    pub fn bigger_equal_than(&self, xy: XY) -> bool {
        self.x.map(|x| x >= xy.x).unwrap_or(true) &&
            self.y.map(|y| y >= xy.y).unwrap_or(true)
    }

    pub fn strictly_bigger_than(&self, xy: XY) -> bool {
        self.x.map(|x| x > xy.x).unwrap_or(true) &&
            self.y.map(|y| y > xy.y).unwrap_or(true)
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_some() && self.y().is_some()
    }

    pub fn is_infinite(&self) -> bool {
        self.x.is_none() || self.y().is_none()
    }

    pub fn as_finite(&self) -> Option<XY> {
        match (self.x, self.y) {
            (Some(x), Some(y)) => Some(XY::new(x, y)),
            _ => None
        }
    }

    /*
    Returns intersection of Rect with self, if nonempty, along with proper visibility hint.
     */
    //TODO tests
    pub fn cut_out_rect(&self, rect: Rect) -> Option<SizeConstraint> {
        // no overlap, rect is too far out x
        if self.x.map(|max_x| max_x < rect.lower_right().x).unwrap_or(false) {
            return None;
        }
        // no overlap, rect is too far out y
        if self.y.map(|max_y| max_y < rect.lower_right().y).unwrap_or(false) {
            return None;
        }

        let new_upper_left = rect.upper_left();
        let mut new_lower_right = rect.lower_right();
        if let Some(max_x) = self.x {
            if new_lower_right.x > max_x {
                new_lower_right.x = max_x;
            }
        }
        if let Some(max_y) = self.y {
            if new_lower_right.y > max_y {
                new_lower_right.y = max_y;
            }
        }

        // if let Some(new_visible_rect) = self.visible.intersect(&rect) {
        if new_lower_right > new_upper_left {
            let mut new_size = new_lower_right - new_upper_left;
            let new_rect = Rect::new(new_upper_left, new_lower_right - new_upper_left);
            Some(
                // SizeConstraint::new(Some(new_size.x), Some(new_size.y), new_visible_rect)
                SizeConstraint::new(Some(new_size.x), Some(new_size.y), new_rect)
            )
        } else {
            debug!("empty intersection of rect {:?} and sc {:?}", &rect, self);
            None
        }
    }

    /*
    Returns SizeConstraint that comes from this one by applying symmetrical margins, if nonempty, along with proper visibility hint.
     */
    //TODO tests
    pub fn cut_out_margin(&self, margin: XY) -> Option<SizeConstraint> {
        if self.x.map(|max_x| max_x < 1 + margin.x * 2).unwrap_or(false) {
            debug!("not enough x");
            return None;
        }
        if self.y.map(|max_y| max_y < 1 + margin.y * 2).unwrap_or(false) {
            debug!("not enough y");
            return None;
        }

        let mut new_rect_upper_left = self.visible.upper_left();
        if new_rect_upper_left.x < margin.x {
            new_rect_upper_left.x = margin.x;
        }
        if new_rect_upper_left.y < margin.y {
            new_rect_upper_left.y = margin.y;
        }

        let mut new_rect_lower_right = self.visible.lower_right();
        if let Some(max_x) = self.x {
            new_rect_lower_right.x = min(new_rect_lower_right.x, max_x - margin.x);
        }
        if let Some(max_y) = self.y {
            new_rect_lower_right.y = min(new_rect_lower_right.y, max_y - margin.y);
        }

        let new_visible_rect = if new_rect_lower_right > new_rect_upper_left {
            Rect::new(new_rect_upper_left, new_rect_lower_right - new_rect_upper_left)
        } else {
            debug!("visible part would be empty/deformed");
            return None;
        };

        Some(SizeConstraint::new(
            self.x.map(|old_x| old_x - (margin.x * 2)),
            self.y.map(|old_y| old_y - (margin.y * 2)),
            new_visible_rect,
        ))
    }

    /*
    This function removes (substracts) top rows / left columns, limiting "visible hint" when
    necessary. It's used  for layouting */
    pub fn substract(&self, xy: XY) -> Option<SizeConstraint> {
        let new_x = if let Some(x) = self.x {
            if x <= xy.x {
                return None;
            };
            Some(x - xy.x)
        } else {
            None
        };

        let new_y = if let Some(y) = self.y {
            if y <= xy.y {
                return None;
            };
            Some(y - xy.y)
        } else {
            None
        };

        let mut new_rect_upper_left = self.visible.upper_left();
        new_rect_upper_left.x = min(new_rect_upper_left.x, xy.x);
        new_rect_upper_left.y = min(new_rect_upper_left.y, xy.y);

        let mut new_rect_lower_right = self.visible.lower_right();
        new_rect_lower_right.x = min(new_rect_lower_right.x, xy.x);
        new_rect_lower_right.y = min(new_rect_lower_right.y, xy.y);

        if new_rect_lower_right > new_rect_upper_left {
            let new_visible_rect = Rect::new(new_rect_upper_left, new_rect_lower_right - new_rect_upper_left);
            Some(SizeConstraint::new(new_x, new_y, new_visible_rect))
        } else {
            debug!("not returning new sc, visible rect is none");
            None
        }
    }
}