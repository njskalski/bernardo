// some would call this a "modal" layout, that is:
// there's a background layout and foreground layout.
// Background is visible but inactive, foreground is visible.

use log::error;

use crate::{Output, Theme, Widget};
use crate::layout::layout::{Layout, WidgetIdRect, WidgetWithRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

pub struct HoverLayout<W: Widget> {
    parent: Box<dyn Layout<W>>,
    child: Box<dyn Layout<W>>,
    child_rect: Rect,
}

impl<W: Widget> HoverLayout<W> {
    pub fn new(parent: Box<dyn Layout<W>>, child: Box<dyn Layout<W>>, child_rect: Rect) -> Self {
        //TODO handle child bigger than parent

        HoverLayout {
            parent,
            child,
            child_rect,
        }
    }
}

impl<W: Widget> Layout<W> for HoverLayout<W> {
    fn min_size(&self, root: &W) -> XY {
        self.parent.min_size(root)
    }

    fn calc_sizes(&self, root: &mut W, output_size: XY) -> Vec<WidgetIdRect> {
        let mut result = self.parent.calc_sizes(root, output_size);

        if !(output_size > self.child_rect.lower_right()) {
            error!("not enough space to draw child {} at {}", self.child_rect, output_size);
        } else {
            let mut partial: Vec<WidgetIdRect> = self.child.calc_sizes(root, self.child_rect.size).iter_mut().map(
                |wir| wir.shifted(self.child_rect.pos)
            ).collect();

            result.append(&mut partial);
        }

        result
    }

    fn layout(&self, root: &mut W, output_size: XY) -> Vec<WidgetWithRect<W>> {
        let mut result = self.parent.layout(root, output_size);

        if !(output_size > self.child_rect.lower_right()) {
            error!("not enough space to draw child {} at {}", self.child_rect, output_size);
        } else {
            let mut partial: Vec<WidgetWithRect<W>> = self.child.layout(root, self.child_rect.size).into_iter().map(
                |wir| wir.shifted(self.child_rect.pos)
            ).collect();

            result.append(&mut partial);
        }

        result
    }
}