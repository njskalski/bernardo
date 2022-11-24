use std::cmp::{max, min};
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
    As in all cases, returns new SizeConstraing in IT's OWN SPACE, not in self's space.
    TODO actually now it does something else because otherwise shitload of tests fail adn I don't know hwy
     */
    pub fn cut_out_rect(&self, rect: Rect) -> Option<SizeConstraint> {
        // no overlap, rect is too far out x
        if self.x.map(|max_x| max_x < rect.upper_left().x).unwrap_or(false) {
            return None;
        }
        // no overlap, rect is too far out y
        if self.y.map(|max_y| max_y < rect.upper_left().y).unwrap_or(false) {
            return None;
        }

        let new_upper_left = rect.upper_left();
        let mut new_lower_right = rect.lower_right();
        if let Some(max_x) = self.x {
            new_lower_right.x = min(new_lower_right.x, max_x);
        }
        if let Some(max_y) = self.y {
            new_lower_right.y = min(new_lower_right.y, max_y);
        }

        if new_lower_right > new_upper_left {
            // this is in new space
            let new_size = new_lower_right - new_upper_left;
            let new_rect = Rect::new(XY::ZERO, new_size);

            // ok, now I will be shifting visible rect to new space, so moving it by -rect.pos
            let mut new_visible_rect_lower_right = self.visible.lower_right();
            if new_visible_rect_lower_right.x >= rect.pos.x {
                new_visible_rect_lower_right.x -= rect.pos.x;
            } else {
                new_visible_rect_lower_right.x = 0;
            }
            if new_visible_rect_lower_right.y >= rect.pos.y {
                new_visible_rect_lower_right.y -= rect.pos.y;
            } else {
                new_visible_rect_lower_right.y = 0;
            }
            let mut new_visible_rect_upper_left = new_visible_rect_lower_right;
            if new_visible_rect_upper_left.x >= rect.size.x {
                new_visible_rect_upper_left.x -= rect.size.x;
            } else {
                new_visible_rect_upper_left.x = 0;
            }
            if new_visible_rect_upper_left.y >= rect.size.y {
                new_visible_rect_upper_left.y -= rect.size.y;
            } else {
                new_visible_rect_upper_left.y = 0;
            }

            if new_visible_rect_upper_left < new_visible_rect_lower_right {
                Some(
                    SizeConstraint::new(Some(new_size.x), Some(new_size.y),
                                        Rect::new(new_visible_rect_upper_left,
                                                  new_visible_rect_lower_right - new_visible_rect_upper_left,
                                        ))
                )
            } else {
                error!("new visible rect empty");
                None
            }
        } else {
            error!("empty intersection of rect {:?} and sc {:?}", &rect, self);
            None
        }
    }

    /*
    Returns SizeConstraint that comes from this one by applying symmetrical margins, if nonempty, along with proper visibility hint.
    Returns SC in new space (so margin rows and columns with lowest indices are disappeared), and everything is shifted by -margin.
     */
    pub fn cut_out_margin(&self, margin: XY) -> Option<SizeConstraint> {
        if self.x.map(|max_x| max_x < 1 + margin.x * 2).unwrap_or(false) {
            debug!("not enough x");
            return None;
        }
        if self.y.map(|max_y| max_y < 1 + margin.y * 2).unwrap_or(false) {
            debug!("not enough y");
            return None;
        }

        // in "self" space
        let mut new_rect_upper_left = self.visible.upper_left();
        if new_rect_upper_left.x < margin.x {
            new_rect_upper_left.x = margin.x;
        }
        if new_rect_upper_left.y < margin.y {
            new_rect_upper_left.y = margin.y;
        }
        let mut new_rect_lower_right = self.visible.lower_right();
        // I should test for max_x/y > margin.x/y, but condition on top is sufficient.
        if let Some(max_x) = self.x {
            debug_assert!(max_x > margin.x);
            new_rect_lower_right.x = min(new_rect_lower_right.x, max_x - margin.x);
        }
        if let Some(max_y) = self.y {
            debug_assert!(max_y > margin.y);
            new_rect_lower_right.y = min(new_rect_lower_right.y, max_y - margin.y);
        }

        let new_visible_rect = if new_rect_lower_right > new_rect_upper_left {
            Rect::new(new_rect_upper_left - margin, new_rect_lower_right - new_rect_upper_left)
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
    necessary. It's used  for layouting. The new SC is translated by given xy. */
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

        // first, let's construct new visibility rect in space of self
        let mut new_rect_upper_left = self.visible.upper_left();
        new_rect_upper_left.x = max(new_rect_upper_left.x, xy.x);
        new_rect_upper_left.y = max(new_rect_upper_left.y, xy.y);
        // this does not move
        let new_rect_lower_right = self.visible.lower_right();

        if new_rect_lower_right > new_rect_upper_left {
            let mut new_visibility_rect = Rect::new(new_rect_upper_left, new_rect_lower_right - new_rect_upper_left);
            // now let's translate it to the new space
            debug_assert!(new_visibility_rect.pos >= xy);
            new_visibility_rect.pos = new_visibility_rect.pos - xy;
            Some(SizeConstraint::new(new_x, new_y, new_visibility_rect))
        } else {
            debug!("not returning new sc, visible rect is none");
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::primitives::rect::Rect;
    use crate::primitives::size_constraint::SizeConstraint;
    use crate::primitives::xy::XY;

    #[test]
    fn test_cut_out_margin() {
        assert_eq!(
            SizeConstraint::new(None, None, Rect::new(XY::ZERO, XY::new(10, 6))).cut_out_margin(XY::new(2, 1)),
            Some(SizeConstraint::new(None, None, Rect::new(XY::ZERO, XY::new(8, 5)))),
        );

        assert_eq!(
            SizeConstraint::new(Some(10), Some(10), Rect::new(XY::ZERO, XY::new(10, 6))).cut_out_margin(XY::new(2, 1)),
            Some(SizeConstraint::new(Some(6), Some(8), Rect::new(XY::ZERO, XY::new(6, 5)))),
        );
    }

    #[test]
    fn test_substract() {
        assert_eq!(
            SizeConstraint::new(None, None, Rect::new(XY::ZERO, XY::new(10, 6))).substract(XY::new(1, 1)),
            Some(SizeConstraint::new(None, None, Rect::new(XY::ZERO, XY::new(9, 5)))),
        );

        assert_eq!(
            SizeConstraint::new(Some(10), None, Rect::new(XY::ZERO, XY::new(10, 6))).substract(XY::new(2, 1)),
            Some(SizeConstraint::new(Some(8), None, Rect::new(XY::ZERO, XY::new(8, 5)))),
        );

        assert_eq!(
            SizeConstraint::new(Some(10), None, Rect::new(XY::ZERO, XY::new(10, 6))).substract(XY::new(10, 10)),
            None,
        );
    }

    #[test]
    fn test_cut_out_rect() {
        assert_eq!(
            SizeConstraint::new(None, None, Rect::new(XY::ZERO, XY::new(10, 6)))
                .cut_out_rect(Rect::new(XY::new(0, 3), XY::new(6, 5))),
            Some(SizeConstraint::new(Some(6), Some(5), Rect::new(XY::ZERO, XY::new(6, 3)))),
        );
    }
}