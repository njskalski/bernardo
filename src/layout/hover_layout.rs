// some would call this a "modal" layout, that is:
// there's a background layout and foreground layout.
// Background is visible but inactive, foreground is visible.

use log::{error, warn};

use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub type ChildRectFunc = Box<dyn Fn(SizeConstraint) -> Option<Rect>>;

pub struct HoverLayout<W: Widget> {
    parent: Box<dyn Layout<W>>,
    child: Box<dyn Layout<W>>,

    child_rect_func: ChildRectFunc,

    blocking_background: bool,
}

impl<W: Widget> HoverLayout<W> {
    pub fn new(parent: Box<dyn Layout<W>>, child: Box<dyn Layout<W>>, child_rect_func: ChildRectFunc, blocking_background: bool) -> Self {
        //TODO handle child bigger than parent

        HoverLayout {
            parent,
            child,
            child_rect_func,
            blocking_background,
        }
    }
}

impl<W: Widget> Layout<W> for HoverLayout<W> {
    fn prelayout(&self, root: &mut W) {
        self.parent.prelayout(root);
        self.child.prelayout(root);
    }

    fn min_size(&self, root: &W) -> XY {
        self.parent.min_size(root)
    }

    fn layout(&self, root: &mut W, sc: SizeConstraint) -> LayoutResult<W> {
        let mut result = self.parent.layout(root, sc);

        if self.blocking_background {
            for wwr in result.wwrs.iter_mut() {
                wwr.set_focusable(false);
            }
        }

        let child_rect_op = (self.child_rect_func)(sc);
        if let Some(child_rect) = child_rect_op {
            if !(sc.bigger_equal_than(child_rect.lower_right())) {
                error!("not enough space to draw child {} at {}", child_rect, sc);
            } else {
                if let Some(new_sc) = sc.cut_out_rect(child_rect) {
                    let mut partial: Vec<WidgetWithRect<W>> = self.child.layout(root, new_sc).wwrs.into_iter().map(
                        |wir| wir.shifted(child_rect.pos)
                    ).collect();

                    result.wwrs.append(&mut partial);
                }
            }
        } else {
            warn!("no child rec in hover layout");
        }

        result
    }
}