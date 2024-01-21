use log::error;

use crate::experiments::screenspace::Screenspace;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct FrameLayout<W: Widget> {
    layout: Box<dyn Layout<W>>,
    margins: XY,
}

impl<W: Widget> FrameLayout<W> {
    pub fn new(layout: Box<dyn Layout<W>>, margins: XY) -> Self {
        Self { layout, margins }
    }

    pub fn sub_rect(&self, size: XY) -> Option<Rect> {
        if size > self.margins * 2 {
            Some(Rect::new(self.margins, size - (self.margins * 2)))
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
        if !(self.margins * 2 < output_size) {
            error!("output size not twice margin size");
            return output_size;
        }

        self.layout.exact_size(root, output_size - (self.margins * 2)) + self.margins * 2
    }

    fn layout(&self, root: &mut W, screenspace: Screenspace) -> LayoutResult<W> {
        if !(self.margins * 2 < screenspace.output_size()) {
            error!("output size not twice margin size");
            return LayoutResult::new(Vec::default(), screenspace.output_size());
        }

        if let Some(sub_rect) = self.sub_rect(screenspace.output_size()) {
            let new_visible_rect_parent_size = sub_rect.intersect(screenspace.visible_rect()).unwrap(); //TODO
            let mut new_visible_rect = new_visible_rect_parent_size;
            new_visible_rect.pos = XY::ZERO;

            let new_output_size = sub_rect.size;
            let subresp = self.layout.layout(root, Screenspace::new(new_output_size, new_visible_rect));

            let wwrs: Vec<WidgetWithRect<W>> = subresp.wwrs.into_iter().map(|wir| wir.shifted(sub_rect.pos)).collect();

            LayoutResult::new(wwrs, subresp.total_size + self.margins * 2)
        } else {
            error!("no visible rect, skipping entire layout");

            LayoutResult::new(Vec::default(), screenspace.output_size())
        }
    }
}
