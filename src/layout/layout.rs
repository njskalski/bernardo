use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::experiments::focus_group::FocusUpdate;
use crate::widget::widget::wid;
use std::iter::Iterator;

pub trait Layout {
    fn get_focused(&self) -> usize;

    // result == true if focus got changed, false otherwise. It's for sound or visualization.
    fn update_focus(&mut self, focus_update : &FocusUpdate) -> bool;

    fn get_rect(&self, output_size: XY, widget_id: wid) -> Option<Rect>;

    fn is_leaf(&self) -> bool;

    fn has_id(&self, widget_id: wid) -> bool;

    //TODO this should not be a vec, should be iter.
    fn get_ids(&self) -> Vec<wid>;

    fn get_all(&self, output_size: XY) -> Vec<(wid, Option<Rect>)> {
        self.get_ids().iter().map(|wid| (*wid, self.get_rect(output_size, *wid))).collect()
    }
}
