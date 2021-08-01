use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

pub enum FocusUpdate {
    Left,
    Right,
    Up,
    Down,
    Next,
    Prev,
}

pub trait Layout {
    fn get_focused(&self) -> usize;

    // result == true if focus got changed, false otherwise. It's for sound or visualization.
    fn update_focus(&mut self, focus_update : FocusUpdate) -> bool;

    fn get_rect(&self, output_size: XY, widget_id: usize) -> Option<Rect>;
}
