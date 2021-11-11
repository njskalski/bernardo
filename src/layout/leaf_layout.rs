use std::slice::Iter;

use log::warn;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::layout::layout::{Layout, WidgetGetter, WidgetGetterMut, WidgetIdRect, WidgetRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::{WID, Widget};

pub struct LeafLayout<'a> {
    widget: &'a dyn Widget,
}

impl<'a> LeafLayout<'a> {
    pub fn new(widget: &'a dyn Widget) -> Self {
        LeafLayout { widget }
    }
}

impl<'a> Layout<'a> for LeafLayout<'a> {
    fn is_leaf(&self) -> bool {
        true
    }

    fn min_size(&self) -> XY {
        self.widget.min_size()
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetRect> {
        let widget = self.widget.clone();

        let size = self.widget.layout(output_size);
        let rect = Rect::new(ZERO, size);

        vec![WidgetRect::new(
            widget,
            rect,
        )]
    }
}
