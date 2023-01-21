use log::error;

use crate::layout::layout::{Layout, LayoutResult};
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

// This layout exists only to "fill"

pub struct EmptyLayout {
    size: Option<XY>,
}

impl EmptyLayout {
    pub fn new() -> Self {
        Self {
            size: None,
        }
    }

    pub fn with_size(self, size: XY) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl<W: Widget> Layout<W> for EmptyLayout {
    fn prelayout(&self, root: &mut W) {}

    fn min_size(&self, _root: &W) -> XY {
        self.size.unwrap_or(XY::ZERO)
    }

    fn layout(&self, _root: &mut W, sc: SizeConstraint) -> LayoutResult<W> {
        let viewport_size: XY = match (sc.as_finite(), self.size) {
            (Some(sc), Some(size)) => {
                if sc < size {
                    error!("layouting EmptyLayout with not enough space!");
                };
                sc
            }
            (Some(sc), None) => sc,
            (None, Some(size)) => size,
            (None, None) => {
                error!("requested empty layout size with no size set and no constraint on input. Returning (1,1).");
                XY::new(1, 1)
            }
        };

        LayoutResult::new(Vec::default(), viewport_size)
    }
}
