use crate::io::input_event::InputEvent;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait MsgConstraints : Copy + Clone + Debug {}

pub trait BaseWidget {
    fn id(&self) -> usize;
}

pub trait Widget<ParentMsg: MsgConstraints> : BaseWidget {
    type LocalMsg;

    fn update(&mut self, msg: Self::LocalMsg) -> Option<ParentMsg>;

    fn focusable(&self) -> bool;

    /*
    returns Some() if event was consumed.
    separated from update, so the InputEvent can get escalated.
     */
    fn on_input(&self, input_event: InputEvent) -> Option<Self::LocalMsg>;
}

pub fn get_new_widget_id() -> usize {
    static COUNTER:AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub const WIDGET_NONE: usize = 0;