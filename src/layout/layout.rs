use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

pub type WidgetGetter<T> = Box<dyn Fn(&'_ T) -> &'_ dyn Widget>;
pub type WidgetGetterMut<T> = Box<dyn Fn(&'_ mut T) -> &'_ mut dyn Widget>;

// TODO I want to get to the point where all layout is generated from macros, and then
// depending on whether root is mut or not, we get mut layout or not-mut layout.


#[derive(Clone, Copy, Debug)]
pub struct WidgetIdRect {
    pub wid: WID,
    pub rect: Rect,
}

impl WidgetIdRect {
    pub fn new(wid: WID, rect: Rect) -> Self {
        WidgetIdRect {
            wid,
            rect,
        }
    }

    pub fn shifted(self, pos: XY) -> Self {
        WidgetIdRect {
            wid: self.wid,
            rect: self.rect.shifted(pos),
        }
    }
}

pub trait Layout<W: Widget> {
    fn min_size(&self, root: &W) -> XY;

    /*
    This only calculates the rects under current constraints. The widgets themselves should
    receive information about their new sizes before render.
     */
    fn calc_sizes(&self, root: &mut W, output_size: XY) -> Vec<WidgetIdRect>;

    fn boxed(self) -> Box<dyn Layout<W>> where Self: Sized, Self: 'static {
        Box::new(self)
    }
}
