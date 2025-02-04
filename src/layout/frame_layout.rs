use log::error;

use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::frame_widget::FrameWidget;

pub struct FrameLayout<W: Widget> {
    layout: Box<dyn Layout<W>>,
    margins: XY,
    frame: Option<SubwidgetPointer<W>>,
}

impl<W: Widget> FrameLayout<W> {
    pub fn new(layout: Box<dyn Layout<W>>, margins: XY) -> Self {
        Self {
            layout,
            margins,
            frame: None,
        }
    }

    pub fn sub_rect(&self, size: XY) -> Option<Rect> {
        if size > self.margins * 2 {
            Some(Rect::new(self.margins, size - (self.margins * 2)))
        } else {
            None
        }
    }

    pub fn with_frame(self, frame: SubwidgetPointer<W>) -> Self {
        Self {
            frame: Some(frame),
            ..self
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

            let mut wwrs: Vec<WidgetWithRect<W>> = subresp.wwrs.into_iter().map(|wir| wir.shifted(sub_rect.pos)).collect();

            if let Some(frame) = self.frame.clone() {
                let frame_widget = frame.get_mut(root);
                frame_widget.layout(screenspace.clone());
                wwrs.insert(
                    0,
                    WidgetWithRect::new(frame, Rect::from_zero(screenspace.output_size()), frame_widget.is_focusable()),
                );
            }

            LayoutResult::new(wwrs, subresp.total_size + self.margins * 2)
        } else {
            error!("no visible rect, skipping entire layout");

            LayoutResult::new(Vec::default(), screenspace.output_size())
        }
    }
}
