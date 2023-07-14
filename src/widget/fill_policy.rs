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
pub struct SizePolicy {
    pub x: DeterminedBy,
    pub y: DeterminedBy,
}

impl SizePolicy {
    pub const SELF_DETERMINED: SizePolicy = SizePolicy {
        x: DeterminedBy::Widget,
        y: DeterminedBy::Widget,
    };

    pub const MATCH_LAYOUTS_WIDTH: SizePolicy = SizePolicy {
        x: DeterminedBy::Layout,
        y: DeterminedBy::Widget,
    };

    pub const MATCH_LAYOUTS_HEIGHT: SizePolicy = SizePolicy {
        x: DeterminedBy::Widget,
        y: DeterminedBy::Layout,
    };

    pub const MATCH_LAYOUT: SizePolicy = SizePolicy {
        x: DeterminedBy::Layout,
        y: DeterminedBy::Layout,
    };
}

impl Default for SizePolicy {
    fn default() -> Self {
        SizePolicy::SELF_DETERMINED
    }
}
