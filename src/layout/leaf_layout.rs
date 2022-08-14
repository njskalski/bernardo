use log::warn;

use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::Widget;

pub struct LeafLayout<W: Widget> {
    widget: SubwidgetPointer<W>,
    with_border: bool,
}

impl<W: Widget> LeafLayout<W> {
    pub fn new(widget: SubwidgetPointer<W>) -> Self {
        LeafLayout { widget, with_border: false }
    }

    pub fn with_border(self) -> Self {
        LeafLayout {
            with_border: true,
            ..self
        }
    }
}

impl<W: Widget> Layout<W> for LeafLayout<W> {
    fn min_size(&self, root: &W) -> XY {
        self.widget.get(root).min_size()
    }

    fn calc_sizes(&self, root: &mut W, output_size: XY) -> Vec<WidgetIdRect> {
        let wid = self.widget.get(root).id();

        let res = if self.with_border {
            if output_size > (2, 2).into() {
                let limited_output = XY::new(output_size.x - 2, output_size.y - 2);
                let size = self.widget.get_mut(root).layout(SizeConstraint::simple(limited_output));
                let rect = Rect::new(XY::new(1, 1), size);

                vec![WidgetIdRect {
                    wid,
                    rect,
                }]
            } else {
                warn!("too small LeafLayout to draw the view.");
                vec![]
            }
        } else {
            let size = self.widget.get_mut(root).layout(SizeConstraint::simple(output_size));
            let rect = Rect::new(ZERO, size);

            vec![WidgetIdRect {
                wid,
                rect,
            }]
        };

        for wid in &res {
            debug_assert!(output_size >= wid.rect.lower_right());
        }

        res
    }
}

