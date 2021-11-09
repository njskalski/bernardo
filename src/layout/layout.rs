use std::iter::Iterator;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

pub type WidgetGetter<T: Widget> = Box<dyn Fn(&'_ T) -> &'_ dyn Widget>;
pub type WidgetGetterMut<T: Widget> = Box<dyn Fn(&'_ mut T) -> &'_ mut dyn Widget>;

pub struct WidgetIdRect {
    pub wid: WID,
    pub rect: Rect,
}

pub trait Layout<W: Widget> {
    fn is_leaf(&self) -> bool {
        false
    }

    fn min_size(&self, owner: &W) -> XY;

    /*
    This is guaranteed to be called before render.
     */
    fn sizes(&mut self, owner: &W, output_size: XY) -> Vec<WidgetIdRect>;


    fn render(&self, owner: &W, focused_id: Option<WID>, output: &mut Output);
    //
    // fn boxed(self) -> Box<dyn Layout<W>> where Self: Sized {
    //     Box::new(self)
    // }
}
