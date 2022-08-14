use crate::{Output, Theme, Widget};
use crate::layout::layout::{Layout, WidgetIdRect, WidgetWithRect};
use crate::primitives::xy::{XY, ZERO};

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
    fn min_size(&self, root: &W) -> XY {
        self.size.unwrap_or(ZERO)
    }

    fn layout(&self, root: &mut W, output_size: XY) -> Vec<WidgetWithRect<W>> {
        Vec::default()
    }
}
