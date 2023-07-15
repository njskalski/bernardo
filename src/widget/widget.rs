use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::SizePolicy;

// this corresponds to message to Parent.
pub type WidgetAction<W> = fn(&W) -> Option<Box<dyn AnyMsg>>;
pub type WidgetActionParam<W, P> = fn(&W, P) -> Option<Box<dyn AnyMsg>>;

pub type WID = usize;

pub trait Widget: 'static {
    fn id(&self) -> WID;

    fn typename(&self) -> &'static str;

    // This call is created to pull widget-independent data before size() call.
    // TODO replace with subscriptions?
    fn prelayout(&mut self) {}

    // Size of the view. If the output cannot satisfy it, a replacement is drawn instead, and the
    // view cannot be focused.
    //
    // Size can be data dependent. Widgets *should not* implement their own scrolling.
    // So it's perfectly reasonable for widget to require O(n) time to answer question "how big I
    // need to be" (if it's slow, we'll be caching *after* profiling).
    fn full_size(&self) -> XY;

    // This is information to layout on how the Widget wants to be drawn. It's completely optional
    fn size_policy(&self) -> SizePolicy { SizePolicy::SELF_DETERMINED }

    // Invariants:
    // - visible_rect is not empty and not degraded (we don't layout invisible stuff)
    // - full_size <= output_size (if we can't satisfy requirement, we don't draw)
    fn layout(&mut self, output_size: XY, visible_rect: Rect);

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

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output);


    // Kite is the part of view that is supposed to be followed by the scroll. Scroll always makes
    // the least amount of movement so the display contain a kite.
    //
    // Why the name? Well, I was looking for something "opposite to an anchor".
    fn kite(&self) -> XY {
        XY::ZERO
    }

    fn as_any(&self) -> &dyn Widget where Self: Sized {
        self as &dyn Widget
    }

    fn as_any_mut(&mut self) -> &mut dyn Widget where Self: Sized {
        self as &mut dyn Widget
    }
}

impl<'a> Debug for dyn Widget + 'a {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "W{}:{}", self.typename(), self.id())
    }
}

impl<'a> Display for dyn Widget + 'a {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "W{}:{}", self.typename(), self.id())
    }
}

pub fn get_new_widget_id() -> WID {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed) as WID
}

pub const WIDGET_NONE: usize = 0;
