use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::action_trigger::ActionTrigger;
use crate::widget::any_msg::AnyMsg;

// this corresponds to message to Parent.
pub type WidgetAction<W> = fn(&W) -> Option<Box<dyn AnyMsg>>;
pub type WidgetActionParam<W, P> = fn(&W, P) -> Option<Box<dyn AnyMsg>>;

pub type WID = usize;

pub trait Widget: 'static {
    fn id(&self) -> WID;

    fn typename(&self) -> &'static str;

    // Minimal size of the view. If the output cannot satisfy it, a replacement is drawn instead,
    // and the view cannot be focused.
    //
    // min_size can be data dependent. Widgets *should not* implement their own scrolling.
    // So it's perfectly reasonable for widget to require O(n) time to answer question "how big I
    // need to be" (if it's slow, we'll be caching *after* profiling).
    fn min_size(&self) -> XY;

    // This is guaranteed to be called before each render. It is guaranteed to be called with
    // SizeConstraint >= self.min_size().
    //
    // This is an opportunity for widget to "update itself" and decide how it's going to be drawn.
    // There is no enforced contract on whether widget should layout it's subwidgets first or
    // afterwards, or if even at all. A widget can decide to remove child widget and *not* layout it
    // for whatever reason.
    //
    // Widget is given size constraint and returns "how much space I will use under given constraints".
    // Whether widget "fills" the space or just uses as little as it can depends on Widget, not on
    // layout. No css bs here.
    //
    // It is assumed that no widget is "infinite", because I say so. Infinite sources are not
    // supported at this time.
    //
    // In case I forget why I added it: to inform the "split layout" on actual size of widgets.
    // Without it, it would be impossible to decide "which widget gets how much space" before
    // rendering them.
    //
    // If widget size is VisibleRect dependent (everything that fills buffers in greedy way) and
    // SizeConstraint.visible_rect() == None, a self.min_size() is used instead.
    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY;


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

    fn anchor(&self) -> XY {
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

pub fn get_new_widget_id() -> WID {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed) as WID
}

pub const WIDGET_NONE: usize = 0;
