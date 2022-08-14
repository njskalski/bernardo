use crate::layout::layout::{Layout, WidgetIdRect};
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

impl Layout for EmptyLayout {
    fn min_size(&self) -> XY {
        self.size.unwrap_or(ZERO)
    }

    fn calc_sizes(&mut self, _output_size: XY) -> Vec<WidgetIdRect> {
        vec![]
    }
}
