use log::warn;



use crate::layout::layout::{Layout, WidgetIdRect};


use crate::primitives::rect::Rect;

use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::{Widget};

pub struct LeafLayout<'a> {
    widget: &'a mut dyn Widget,
    with_border: bool,
}

impl<'a> LeafLayout<'a> {
    pub fn new(widget: &'a mut dyn Widget) -> Self {
        LeafLayout { widget, with_border: false }
    }

    pub fn with_border(self) -> Self {
        LeafLayout {
            with_border: true,
            ..self
        }
    }
}

impl<'a> Layout for LeafLayout<'a> {
    fn is_leaf(&self) -> bool {
        true
    }

    fn min_size(&self) -> XY {
        self.widget.min_size()
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        let wid = self.widget.id();

        if self.with_border {
            if output_size > (2, 2).into() {
                let limited_output = XY::new(output_size.x - 2, output_size.y - 2);
                let size = self.widget.layout(limited_output);
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
            let size = self.widget.layout(output_size);
            let rect = Rect::new(ZERO, size);

            vec![WidgetIdRect {
                wid,
                rect,
            }]
        }
    }
}
