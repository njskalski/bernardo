use log::{error, warn};

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
        Some(Rect::new(XY::ZERO, output_size))
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

        let properly_sized = sc.bigger_equal_than(widget.min_size());

        if !skip {
            if properly_sized {
                let xy = widget.update_and_layout(sc);
                vec![WidgetWithRect::new(
                    self.widget.clone(),
                    Rect::new(XY::ZERO, xy),
                    true,
                )]
            } else {
                error!("got layout too small to draw widget [{:?}]", widget);
                vec![]
            }
        } else {
            vec![]
        }
    }
}

