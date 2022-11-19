use log::warn;

use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::layout::Layout;
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct LeafLayout<W: Widget> {
    widget: SubwidgetPointer<W>,
}

impl<W: Widget> LeafLayout<W> {
    pub fn new(widget: SubwidgetPointer<W>) -> Self {
        LeafLayout { widget }
    }

    fn rect(&self, output_size: XY) -> Option<Rect> {
        if self.with_border {
            if output_size > (3, 3).into() {
                Some(Rect::new(XY::new(1, 1), XY::new(output_size.x - 2, output_size.y - 2)))
            } else {
                None
            }
        } else {
            Some(Rect::new(XY::ZERO, output_size))
        }
    }
}

impl<W: Widget> Layout<W> for LeafLayout<W> {
    fn min_size(&self, root: &W) -> XY {
        self.widget.get(root).min_size()
    }

    fn layout(&self, root: &mut W, sc: SizeConstraint) -> Vec<WidgetWithRect<W>> {
        let root_id = root.id();
        let widget = self.widget.get_mut(root);
        let skip = root_id == widget.id();

        if !skip {
            let xy = widget.update_and_layout(sc);
            vec![WidgetWithRect::new(
                self.widget.clone(),
                Rect::new(XY::ZERO, xy),
                true,
            )]
        } else {
            vec![]
        }
    }
}

