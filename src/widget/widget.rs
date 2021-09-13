use crate::io::input_event::InputEvent;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::io::output::Output;
use crate::primitives::xy::XY;
use crate::primitives::sized_xy::SizedXY;
use crate::widget::any_msg::AnyMsg;
use crate::layout::leaf_layout::LeafLayout;

// this corresponds to message to Parent.
pub type WidgetAction<W> = fn(&W) -> Option<Box<dyn AnyMsg>>;

pub type WID = usize;

pub trait Widget {
    fn id(&self) -> WID;

    // Minimal size of the view. If the output cannot satisfy it, a replacement is drawn instead,
    // and the view cannot be focused (TODO or input will be ignored, haven't decided that yet).
    fn min_size(&self) -> XY;

    // Size is to be guaranteed to be called with max_size >= min_size.
    fn size(&self, max_size : XY) -> XY;

    // If input is consumed, the output is Some(.). If you don't like it, add noop msg to your widget.
    fn on_input(&self, input_event : InputEvent) -> Option<Box<dyn AnyMsg>>;

    // This is called when an input got consumed and internal message is created.
    // The output is a message to parent.
    // No message will NOT stop redraw.
    fn update(&mut self, msg : Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>>;

    fn get_focused(&self) -> &dyn Widget;
    fn get_focused_mut(&mut self) -> &mut dyn Widget;

    /*
    Each widget has it's widget space, based (0, 0) and sized XY.
    Each output has it's output space, based (0, 0) and sized XY.

    Frame is a Rect based in View's space. But (0,0) of Frame corresponds to (0,0) in output.
    To "simplify" things, I don't pass frame as Rect, I just pass it's beginning, and size
    is deduced from ouput.size().
    Hence the parameter frame_offset, which is frame.pos (2) - view.pos (1) in any space.
    To calculate Frame in widget space, just take Rect::new(frame_offset, output.size()).

    view
    1───────┐
    │       │ frame
    │   2───┼────┐
    │   │   │    │
    │   │   │    │
    │   └───┼────┘
    │       │
    │       │
    └───────┘


     */
    fn render(&self, focused : bool, frame_offset : XY, output : &mut Output);
}

pub fn get_new_widget_id() -> WID {
    static COUNTER:AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed) as WID
}

pub const WIDGET_NONE: usize = 0;