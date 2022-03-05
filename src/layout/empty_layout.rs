use log::warn;

use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::WID;

// This layout exists only to "fill"

pub struct EmptyLayout {}


impl EmptyLayout {
    pub fn new() -> Self {
        Self {}
    }
}

impl Layout for EmptyLayout {
    fn is_leaf(&self) -> bool {
        true
    }

    fn min_size(&self) -> XY {
        XY::new(0, 0)
    }

    fn calc_sizes(&mut self, _output_size: XY) -> Vec<WidgetIdRect> {
        vec![]
    }
}
