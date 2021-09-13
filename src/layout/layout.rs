use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{Widget, WID};
use std::iter::Iterator;

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

    // this two can be merged later.
    fn get_rects(&self, owner: &W, output_size: XY) -> Vec<WidgetIdRect>;
    fn render(&self, owner: &W, focused_id: Option<WID>, output: &mut Output);
}
