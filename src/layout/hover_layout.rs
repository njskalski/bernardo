// some would call this a "modal" layout, that is:
// there's a background layout and foreground layout.
// Background is visible but inactive, foreground is visible.

use log::{error};

use crate::layout::layout::{Layout, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

pub struct HoverLayout<'a> {
    parent: &'a mut dyn Layout,
    child: &'a mut dyn Layout,
    child_rect: Rect,
}

impl<'a> HoverLayout<'a> {
    pub fn new(parent: &'a mut dyn Layout, child: &'a mut dyn Layout, child_rect: Rect) -> Self {
        //TODO handle child bigger than parent

        HoverLayout {
            parent,
            child,
            child_rect,
        }
    }
}

impl<'a> Layout for HoverLayout<'a> {
    fn min_size(&self) -> XY {
        self.parent.min_size()
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        let mut result = self.parent.calc_sizes(output_size);

        if !(output_size > self.child_rect.lower_right()) {
            error!("not enough space to draw child {} at {}", self.child_rect, output_size);
        } else {
            let mut partial: Vec<WidgetIdRect> = self.child.calc_sizes(self.child_rect.size).iter_mut().map(
                |wir| wir.shifted(self.child_rect.pos)
            ).collect();

            result.append(&mut partial);
        }

        result
    }
}