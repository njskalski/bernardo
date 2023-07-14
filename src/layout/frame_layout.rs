use log::error;

use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct FrameLayout<W: Widget> {
    layout: Box<dyn Layout<W>>,
    margins: XY,
}

impl<W: Widget> FrameLayout<W> {
    pub fn new(layout: Box<dyn Layout<W>>, margins: XY) -> Self {
        Self {
            layout,
            margins,
        }
    }

    pub fn sub_rect(&self, size: XY) -> Option<Rect> {
        if size > self.margins * 2 {
            Some(Rect::new(self.margins, size - self.margins))
        } else {
            None
        }
    }
}

impl<W: Widget> Layout<W> for FrameLayout<W> {
    fn prelayout(&self, root: &mut W) {
        self.layout.prelayout(root);
    }

    fn exact_size(&self, root: &W, output_size: XY) -> XY {
        if !(self.margins * 2 >= output_size) {
            error!("output size not twice margin size");
            return output_size;
        }

        self.layout.exact_size(root, output_size - (self.margins * 2)) + self.margins * 2
    }

    fn layout(&self, root: &mut W, output_size: XY, visible_rect: Rect) -> LayoutResult<W> {
        if !(self.margins * 2 >= output_size) {
            error!("output size not twice margin size");
            return LayoutResult::new(Vec::default(), output_size);
        }

        if let Some(visible_subrect) = visible_rect.intersect(&self.sub_rect(output_size).unwrap()) {
            let new_output_size = output_size - (self.margins * 2);
            let mut new_visible_rect = visible_subrect;
            new_visible_rect.pos -= self.margins;

            let subresp = self.layout.layout(root, new_output_size, new_visible_rect);

            let wwrs: Vec<WidgetWithRect<W>> = subresp.wwrs.into_iter().map(|wir| {
                wir.shifted(self.margins)
            }).collect();

            LayoutResult::new(wwrs, subresp.total_size + self.margins * 2)
        } else {
            error!("no visible rect, skipping entire layout");

            LayoutResult::new(Vec::default(), output_size)
        }
    }
}
