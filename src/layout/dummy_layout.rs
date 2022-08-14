use log::warn;

use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::WID;

// This layout exists only so a widget can "layout" itself without referencing itself.
// This is a workaround, I don't know if I want to keep it this way.

pub struct DummyLayout {
    wid: WID,
    size: XY,
    with_border: bool,
}

impl DummyLayout {
    pub fn new(wid: WID, size: XY) -> Self {
        DummyLayout {
            wid,
            size,
            with_border: false,
        }
    }

    pub fn with_border(self) -> Self {
        DummyLayout {
            with_border: true,
            ..self
        }
    }
}

impl Layout for DummyLayout {
    fn min_size(&self) -> XY {
        self.size
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        if self.with_border {
            if output_size > (2, 2).into() {
                let rect = Rect::new(XY::new(1, 1), self.size);

                vec![WidgetIdRect {
                    wid: self.wid,
                    rect,
                }]
            } else {
                warn!("too small LeafLayout to draw the view.");
                vec![]
            }
        } else {
            let rect = Rect::new(ZERO, self.size);

            vec![WidgetIdRect {
                wid: self.wid,
                rect,
            }]
        }
    }
}
