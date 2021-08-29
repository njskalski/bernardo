use crate::io::input_event::InputEvent;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::io::output::Output;
use crate::primitives::xy::XY;
use crate::primitives::sized_xy::SizedXY;
use crate::widget::any_msg::AnyMsg;

// this corresponds to message to Parent.
pub type WidgetAction<W> = fn(&W) -> Option<Box<dyn AnyMsg>>;

pub trait BaseWidget {
    fn id(&self) -> usize;

    fn min_size(&self) -> XY;
    fn size(&self, max_size : XY) -> XY;

    fn on_input_any(&self, input_event : InputEvent) -> Option<Box<dyn AnyMsg>>;

    fn update_any(&mut self, msg : Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>>;

    fn render(&self, focused : bool, output : &mut Output);
}

pub fn get_new_widget_id() -> usize {
    static COUNTER:AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub const WIDGET_NONE: usize = 0;