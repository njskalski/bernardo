use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::experiments::focus_group::FocusUpdate;

pub trait Layout {
    fn get_focused(&self) -> usize;

    // result == true if focus got changed, false otherwise. It's for sound or visualization.
    fn update_focus(&mut self, focus_update : &FocusUpdate) -> bool;

    fn get_rect(&self, output_size: XY, widget_id: usize) -> Option<Rect>;

    fn is_leaf(&self) -> bool;

    fn has_id(&self, widget_id: usize) -> bool;
}
