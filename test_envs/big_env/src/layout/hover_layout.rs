// some would call this a "modal" layout, that is:
// there's a background layout and foreground layout.
// Background is visible but inactive, foreground is visible.

use log::{error, warn};

use crate::experiments::screenspace::Screenspace;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub type ChildRectFunc = Box<dyn Fn(Screenspace) -> Option<Rect>>;

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

    fn exact_size(&self, root: &W, output_size: XY) -> XY {
        self.parent.exact_size(root, output_size)
    }

    fn layout(&self, root: &mut W, screenspace: Screenspace) -> LayoutResult<W> {
        let mut result = self.parent.layout(root, screenspace);

        if self.blocking_background {
            for wwr in result.wwrs.iter_mut() {
                wwr.set_focusable(false);
            }
        }

        let child_rect_op = (self.child_rect_func)(screenspace);
        if let Some(child_rect) = child_rect_op {
            if !(screenspace.output_size() >= child_rect.lower_right()) {
                error!("not enough space to draw child {} within ss {:?}", child_rect, screenspace);
            } else if let Some(child_visible_rect) = screenspace.visible_rect().intersect(child_rect) {
                let mut child_visible_rect_in_child_space = child_visible_rect;
                child_visible_rect_in_child_space.pos -= child_rect.pos;

                let mut partial: Vec<WidgetWithRect<W>> = self
                    .child
                    .layout(root, Screenspace::new(child_rect.size, child_visible_rect_in_child_space))
                    .wwrs
                    .into_iter()
                    .map(|wir| wir.shifted(child_rect.pos))
                    .collect();

                result.wwrs.append(&mut partial);
            }
        } else {
            warn!("no child rec in hover layout");
        }

        result
    }
}
