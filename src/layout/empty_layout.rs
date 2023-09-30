use log::error;

use crate::experiments::screenspace::Screenspace;
use crate::layout::layout::{Layout, LayoutResult};
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
    fn prelayout(&self, _root: &mut W) {}

    fn exact_size(&self, _root: &W, output_size: XY) -> XY {
        self.size.unwrap_or(XY::ZERO)
    }

    fn layout(&self, root: &mut W, screenspace: Screenspace) -> LayoutResult<W> {
        if let Some(requested_size) = self.size {
            if !(requested_size < screenspace.output_size()) {
                error!("requested size {} !< output_size {:?}", requested_size, screenspace);
            }

            LayoutResult::new(Vec::default(), requested_size)
        } else {
            LayoutResult::new(Vec::default(), screenspace.output_size())
        }
    }
}
