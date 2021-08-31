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

    // If input is consumed, the output is Some(.). If you don't like it, add noop msg to your widget.
    fn on_input(&self, input_event : InputEvent) -> Option<Box<dyn AnyMsg>>;

    // This is called when an input got consumed and internal message is created.
    // The output is a message to parent.
    fn update(&mut self, msg : Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>>;

    fn get_focused(&self) -> &dyn BaseWidget;
    fn get_focused_mut(&mut self) -> &mut dyn BaseWidget;

    fn render(&self, focused : bool, output : &mut Output);
}

pub fn get_new_widget_id() -> usize {
    static COUNTER:AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub const WIDGET_NONE: usize = 0;