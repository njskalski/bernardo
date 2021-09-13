use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::experiments::focus_group::FocusUpdate;
use crate::widget::widget::{WID, Widget};
use std::iter::Iterator;
use crate::io::output::Output;

pub type WidgetGetter<T: Widget> = Box<dyn Fn(&'_ T) -> &'_ dyn Widget>;
pub type WidgetGetterMut<T: Widget> = Box<dyn Fn(&'_ mut T) -> &'_ mut dyn Widget>;

pub trait Layout<W : Widget> {
    fn get_focused<'a>(&self, parent: &'a W) -> &'a dyn Widget;
    fn get_focused_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget;

    // result == true if focus got changed, false otherwise. It's for sound or visualization.
    fn update_focus(&mut self, focus_update : FocusUpdate) -> bool;

    // fn get_rect(&self, output_size: XY, widget_id: WID) -> Option<Rect>;

    fn is_leaf(&self) -> bool {
        false
    }

    // fn has_id(&self, widget_id: WID) -> bool;

    //TODO this should not be a vec, should be iter.
    // fn get_ids(&self) -> Vec<WID>;

    // fn get_all(&self, output_size: XY) -> Vec<(WID, Option<Rect>)> {
    //     // self.get_ids().iter().map(|wid| (*wid, self.get_rect( output_size, *wid))).collect()
    //     vec![]
    // }

    fn render(&self, owner : &W, focused_id : Option<WID>, output : &mut Output);
}
