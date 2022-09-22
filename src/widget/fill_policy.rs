/*
FillPolicy is a setting to a widgets that honour it, whether to fill the dimension or not.
Only some widgets will accept it, some will accept only one of the dimensions.

Also, if there is no limit on dimension (scrolling) widget will fall back to "constrained" setup.
 */

use log::warn;

use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FillPolicy {
    pub fill_x: bool,
    pub fill_y: bool,
}

impl FillPolicy {
    pub const CONSTRAINED: FillPolicy = FillPolicy {
        fill_x: false,
        fill_y: false,
    };

    pub const FILL_WIDTH: FillPolicy = FillPolicy {
        fill_x: true,
        fill_y: false,
    };

    pub const FILL_HEIGHT: FillPolicy = FillPolicy {
        fill_x: false,
        fill_y: true,
    };

    pub const FILL_BOTH: FillPolicy = FillPolicy {
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
    pub fn get_size_from_constraints(&self, sc: &SizeConstraint, min_size: XY) -> XY {
        let x = if self.fill_x {
            if let Some(max_x) = sc.x() {
                max_x
            } else {
                warn!("requested to fill an infinite x axis - most probably design bug, falling back to constrained");
                min_size.x
            }
        } else {
            min_size.x
        };

        let y = if self.fill_y {
            if let Some(max_y) = sc.y() {
                max_y
            } else {
                warn!("requested to fill an infinite x axis - most probably design bug, falling back to constrained");
                min_size.y
            }
        } else {
            min_size.y
        };

        XY::new(x, y)
    }
}