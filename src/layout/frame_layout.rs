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

    fn min_size(&self, root: &W) -> XY {
        self.layout.min_size(root) + self.margins * 2
    }

    fn layout(&self, root: &mut W, sc: SizeConstraint) -> LayoutResult<W> {
        if let Some(new_sc) = sc.cut_out_margin(self.margins) {
            let subresp = self.layout.layout(root, new_sc);
            let wwrs: Vec<WidgetWithRect<W>> = subresp.wwrs.into_iter().map(|wir| {
                wir.shifted(self.margins)
            }).collect();

            LayoutResult::new(wwrs, subresp.total_size + self.margins * 2)
        } else {
            error!("too small output to render with margins");

            if let Some(size) = sc.as_finite() {
                LayoutResult::new(Vec::default(), size)
            } else {
                error!("and we don't have a good size estimation. Returning (1,1).");
                LayoutResult::new(Vec::default(), XY::new(1, 1))
            }
        }
    }
}
