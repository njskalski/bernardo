use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{Widget, WID};
use std::iter::Iterator;

pub type WidgetGetter<T: Widget> = Box<dyn Fn(&'_ T) -> &'_ dyn Widget>;
pub type WidgetGetterMut<T: Widget> = Box<dyn Fn(&'_ mut T) -> &'_ mut dyn Widget>;

pub trait Layout<W: Widget> {
    fn get_focused<'a>(&self, parent: &'a W) -> &'a dyn Widget;
    fn get_focused_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget;

    // result == true if focus got changed, false otherwise. It's for sound or visualization.
    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool;

    fn is_leaf(&self) -> bool {
        false
    }

    fn min_size(&self, owner: &W) -> XY;

    fn render(&self, owner: &W, focused_id: Option<WID>, output: &mut Output);
}
