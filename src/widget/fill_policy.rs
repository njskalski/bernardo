/*
FillPolicy is a setting to a widgets that honour it, whether to fill the dimension or not.
Only some widgets will accept it, some will accept only one of the dimensions.

Also, if there is no limit on dimension (scrolling) widget will fall back to "constrained" setup.
 */

use log::debug;

use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DeterminedBy {
    Widget,
    Layout,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FillPolicy {
    pub fill_x: bool,
    pub fill_y: bool,
}

impl FillPolicy {
    pub const SELF_DETERMINED: FillPolicy = FillPolicy {
        fill_x: false,
        fill_y: false,
    };

    pub const MATCH_LAYOUTS_WIDTH: FillPolicy = FillPolicy {
        fill_x: true,
        fill_y: false,
    };

    pub const MATCH_LAYOUTS_HEIGHT: FillPolicy = FillPolicy {
        fill_x: false,
        fill_y: true,
    };

    pub const MATCH_LAYOUT: FillPolicy = FillPolicy {
        fill_x: true,
        fill_y: true,
    };
}

impl Default for FillPolicy {
    fn default() -> Self {
        FillPolicy {
            fill_x: false,
            fill_y: false,
        }
    }
}

impl FillPolicy {
    pub fn get_size_from_constraints(&self, sc: &SizeConstraint, max_size: XY) -> XY {
        let x = if let Some(max_x) = sc.x() {
            if max_size.x > max_x {
                debug!("not enough space on x");
                max_x
            } else {
                if self.fill_x {
                    max_x
                } else {
                    max_size.x
                }
            }
        } else {
            max_size.x
        };

        let y = if let Some(max_y) = sc.y() {
            if max_size.y > max_y {
                debug!("not enough space on y");
                max_y
            } else {
                if self.fill_y {
                    max_y
                } else {
                    max_size.y
                }
            }
        } else {
            max_size.y
        };

        XY::new(x, y)
    }
}