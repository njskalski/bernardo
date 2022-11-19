use log::error;

use crate::layout::layout::Layout;
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
    fn min_size(&self, root: &W) -> XY {
        self.layout.min_size(root) + self.margins * 2
    }

    fn layout(&self, root: &mut W, sc: SizeConstraint) -> Vec<WidgetWithRect<W>> {
        if let Some(new_sc) = sc.cut_out_margin(self.margins) {
            let subs = self.layout.layout(root, new_sc);
            subs.into_iter().map(|wir| {
                wir.shifted(self.margins)
            }).collect()
        } else {
            error!("too small output to render with margins");
            vec![]
        }
    }
}
