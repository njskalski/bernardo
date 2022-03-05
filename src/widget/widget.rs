use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

use log::{debug, error};

use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::any_msg::AnyMsg;

// this corresponds to message to Parent.
pub type WidgetAction<W> = fn(&W) -> Option<Box<dyn AnyMsg>>;

pub type WID = usize;

pub trait Widget {
    fn id(&self) -> WID;

    fn typename(&self) -> &'static str;

    // Minimal size of the view. If the output cannot satisfy it, a replacement is drawn instead,
    // and the view cannot be focused (TODO or input will be ignored, haven't decided that yet).
    fn min_size(&self) -> XY;

    // Description of widget type
    // fn desc() -> &'static str;

    // This is guaranteed to be called before render.
    fn layout(&mut self, sc: SizeConstraint) -> XY;

    // If input is consumed, the output is Some(.). If you don't like it, add noop msg to your widget.
    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>>;

    // This is called when an input got consumed and internal message is created.
    // The output is a message to parent.
    // No message will NOT stop redraw.
    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>>;

    // If these return None, it means that it's not a complex widget - the input should be addressed
    // directly to it.
    // Returning Some(self) would lead to infinite loop.
    fn get_focused(&self) -> Option<&dyn Widget> { None }
    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> { None }

    // Updates focus path from that widget below.
    // Returns whether succeeded.
    fn set_focused(&mut self, wid: WID) -> bool {
        if self.id() == wid {
            true
        } else {
            error!("attempted to update focus_path, but hit non-matching end at widget {}", self.id());
            false
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output);

    fn anchor(&self) -> XY {
        ZERO
    }

    fn subwidgets_mut(&mut self) -> Box<dyn std::iter::Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        debug!("call to default subwidget_mut on {}", self.id());
        Box::new(std::iter::empty())
    }

    fn subwidgets(&self) -> Box<dyn std::iter::Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        debug!("call to default subwidget on {}", self.id());
        Box::new(std::iter::empty())
    }

    fn get_subwidget(&self, wid: WID) -> Option<&dyn Widget> where Self: Sized {
        for widget in self.subwidgets() {
            if widget.id() == wid {
                return Some(widget)
            }
        }

        None
    }

    fn get_subwidget_mut(&mut self, wid: WID) -> Option<&mut dyn Widget> where Self: Sized {
        for widget in self.subwidgets_mut() {
            if widget.id() == wid {
                return Some(widget)
            }
        }

        None
    }
}

impl Debug for dyn Widget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "W{}:{}", self.typename(), self.id())
    }
}

pub fn get_new_widget_id() -> WID {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed) as WID
}

pub const WIDGET_NONE: usize = 0;
