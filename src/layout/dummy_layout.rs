use log::warn;

use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::WID;

// This layout exists only so a widget can "layout" itself without referencing itself.
// This is a workaround, I don't know if I want to keep it this way.

pub struct DummyLayout {
    wid: WID,
    min_size: XY,
}

impl DummyLayout {
    pub fn new(wid: WID, min_size: XY) -> Self {
        DummyLayout {
            wid,
            min_size,
        }
    }

    pub fn with_border(self) -> Self {
        DummyLayout {
            ..self
        }
    }
}

impl Layout for DummyLayout {
    fn min_size(&self) -> XY {
        self.min_size
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        let rect = Rect::new(ZERO, output_size);

            vec![WidgetIdRect {
                wid: self.wid,
                rect,
            }]
    }
}
