use crate::layout::layout::Layout;
use crate::layout::widget_with_rect::WidgetWithRect;
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
    fn min_size(&self, _root: &W) -> XY {
        self.size.unwrap_or(XY::ZERO)
    }

    fn layout(&self, _root: &mut W, _sc: SizeConstraint) -> Vec<WidgetWithRect<W>> {
        Vec::default()
    }
}
