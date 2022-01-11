use log::{warn};


use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
use crate::experiments::scroll::{Scroll, ScrollDirection};
use crate::io::over_output::OverOutput;
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::widget::WID;

pub struct WithScroll<W: Widget> {
    widget: W,
    scroll: Scroll,
}

impl<W: Widget> WithScroll<W> {
    pub fn new(widget: W, scroll_direction: ScrollDirection) -> Self {
        Self {
            widget,
            scroll: Scroll::new(scroll_direction),
        }
    }

    pub fn scroll(&self) -> &Scroll {
        &self.scroll
    }

    fn nested_sc(&self, sc: SizeConstraint) -> SizeConstraint {
        if sc.x().is_none() || sc.y().is_none() {
            warn!("nesting scrolling is not supported - that's beyond TUI")
            // in case of nesting I probably need to add "sc.hint().pos" to offset in Rect
            // constructor below . Or substract it. I don't want to waste mental energy on it now.
        }

        let new_sc = SizeConstraint::new(
            if self.scroll.direction.free_x() { None } else { sc.x() },
            if self.scroll.direction.free_y() { None } else { sc.y() },
            Rect::new(self.scroll.offset, sc.hint().size),
        );

        new_sc
    }
}

impl<W: Widget> Widget for WithScroll<W> {
    fn id(&self) -> WID {
        self.widget.id()
    }

    fn typename(&self) -> &'static str {
        self.widget.typename()
    }

    fn min_size(&self) -> XY {
        self.widget.min_size()
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        let new_sc = self.nested_sc(sc);
        let full_size = self.widget.layout(new_sc);

        // again, in case of nesting I could not just use hint.size
        self.scroll.follow_anchor(sc.hint().size,
                                  self.widget.anchor());

        full_size.cut(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        self.widget.on_input(input_event)
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        self.widget.update(msg)
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.widget.get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.widget.get_focused_mut()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let new_sc = self.nested_sc(output.size_constraint());

        let mut over_output = OverOutput::new(output, new_sc);
        self.widget.render(theme, focused, &mut over_output);
    }

    fn anchor(&self) -> XY {
        // scroll nesting would probably affect that
        ZERO
    }

    fn subwidgets_mut(&mut self) -> Box<dyn Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        self.widget.subwidgets_mut()
    }

    fn subwidgets(&self) -> Box<dyn Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        self.widget.subwidgets()
    }

    fn get_subwidget(&self, wid: WID) -> Option<&dyn Widget> where Self: Sized {
        self.widget.get_subwidget(wid)
    }

    fn get_subwidget_mut(&mut self, wid: WID) -> Option<&mut dyn Widget> where Self: Sized {
        self.get_subwidget_mut(wid)
    }
}