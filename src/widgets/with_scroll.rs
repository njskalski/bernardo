use log::{error, warn};

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::output::{Metadata, Output};
use crate::io::over_output::OverOutput;
use crate::io::sub_output::SubOutput;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::{Scroll, ScrollDirection};
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::unpack_or;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

const DEFAULT_MARGIN_WIDTH: u16 = 4;

pub struct WithScroll<W: Widget> {
    id: WID,
    widget: W,
    scroll: Scroll,
    line_no: bool,

    fill_non_free_axis: bool,
    last_size: Option<XY>,
}

impl<W: Widget> WithScroll<W> {
    pub const TYPENAME: &'static str = "scroll";

    pub fn new(widget: W, scroll_direction: ScrollDirection) -> Self {
        let id = get_new_widget_id();
        Self {
            id,
            widget,
            scroll: Scroll::new(scroll_direction),
            line_no: false,
            fill_non_free_axis: true,
            last_size: None,
        }
    }

    pub fn with_line_no(self) -> Self {
        Self {
            line_no: true,
            ..self
        }
    }

    pub fn scroll(&self) -> &Scroll {
        &self.scroll
    }

    /*
    Returns (margin_width, nested size_constraint).
    This is where it's decided whether to wr
     */
    fn nested_sc(&self, sc: SizeConstraint) -> (u16, SizeConstraint) {
        if sc.x().is_none() || sc.y().is_none() {
            error!("nesting scrolling is not supported - that's beyond TUI")
            // in case of nesting I probably need to add "sc.hint().pos" to offset in Rect
            // constructor below . Or substract it. I don't want to waste mana on it now.
        }

        let margin_width = self.line_count_margin_width(sc);
        let with_margin = self.line_no && sc.strictly_bigger_than(XY::new(margin_width, 0));

        let visible_part_size = sc.visible_hint().map(|visible_hint| if with_margin {
            XY::new(visible_hint.size.x - margin_width, visible_hint.size.y)
        } else {
            visible_hint.size
        });

        let new_sc = SizeConstraint::new(
            if self.scroll.direction.free_x() { None } else {
                sc.x().map(/* this works because strictly_bigger_than above */ |x|
                    if with_margin {
                        x - margin_width
                    } else { x })
            },
            if self.scroll.direction.free_y() { None } else { sc.y() },
            visible_part_size.map(|size| Rect::new(self.scroll.offset, size)),
        );

        (if with_margin { margin_width } else { 0 }, new_sc)
    }

    pub fn internal_mut(&mut self) -> &mut W {
        &mut self.widget
    }

    pub fn internal(&self) -> &W {
        &self.widget
    }

    pub fn mutate_internal<F: FnOnce(W) -> W>(self, mutator: F) -> Self {
        Self {
            widget: mutator(self.widget),
            ..self
        }
    }

    fn line_count_margin_width_for_lower_right(&self, lower_right: XY) -> u16 {
        // logarithm? Never heard of it.
        let width = format!("{}", lower_right.y).len() as u16 + 2;
        width
    }

    fn line_count_margin_width(&self, sc: SizeConstraint) -> u16 {
        /*
        there's a little chicken-egg problem here: to determine width of line_no margin I need to know
        number of lines of self.widget. At the same time, I need this number to decide on layout.

        So here's what I am going to do: I'll take the lower right of scroll.offset + sc.visible_hint,
        to determine margin width and add 1, to make it way more likely that it will not change frame-to-frame.

        I *could* use the previous size, but I want "layout" to NOT use previous state.
         */

        let lower_right = self.scroll.offset +
            sc.visible_hint().map(|vh| vh.lower_right())
                .unwrap_or_else(|| {
                    warn!("layouting scroll without visibility information. Expect bugs.");
                    XY::ZERO
                });
        self.line_count_margin_width_for_lower_right(lower_right)
    }

    fn render_line_no(&self, margin_width: u16, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let size = unpack_or!(self.last_size, (), "render before layout");
        #[cfg(test)]
        {
            output.emit_metadata(Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: Rect::from_zero(XY::new(margin_width, size.y)),
                focused,
            });
        }

        debug_assert!(self.line_no);
        let start_idx = self.scroll.offset.y;

        let style = if focused {
            theme.ui.header
        } else {
            theme.ui.header.half()
        }.with_background(theme.default_text(focused).background);

        // let anchor_row = self.widget.anchor().y;

        for idx in 0..size.y {
            let line_no_base_0 = start_idx + idx;
            let item = format!("{} ", line_no_base_0 + 1);
            let num_digits = item.len() as u16;
            let offset = margin_width - num_digits;

            // let style = if line_no_base_0 == anchor_row {
            //     style.with_background(theme.ui.cursors.background)
            // } else { style };

            for px in 0..offset {
                output.print_at(
                    XY::new(px, idx),
                    style,
                    " ",
                )
            }

            output.print_at(
                XY::new(offset, idx),
                style,
                &item,
            );
        }
    }
}

impl<W: Widget> Widget for WithScroll<W> {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "scroll"
    }

    fn min_size(&self) -> XY {
        let child = self.widget.min_size();
        if self.line_no {
            let margin = self.line_count_margin_width_for_lower_right(child);
            child + XY::new(margin, 0)
        } else {
            child
        }
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        debug_assert!(sc.is_finite(), "nesting scrolls not supported");
        let output_size = if let Some(size) = sc.as_finite() {
            size
        } else {
            error!("nesting scrolls not supported");
            self.widget.min_size()
        };

        let (margin_width, internal_sc) = self.nested_sc(sc);
        let child_size = self.widget.update_and_layout(internal_sc);

        debug_assert!(margin_width >= output_size.x);

        self.scroll.follow_anchor(output_size - XY::new(margin_width, 0),
                                  self.widget.anchor());

        self.last_size = Some(output_size);

        output_size
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
        let parent_size = unpack_or!(self.last_size, (), "render before layout");
        let (margin_width, new_sc) = self.nested_sc(output.size_constraint());

        if margin_width > 0 {
            self.render_line_no(margin_width, theme, focused, output);
        }

        // This is narrowing the scope to make margin for line_no
        let mut sub_output: Option<SubOutput> = if self.line_no {
            let shift = XY::new(margin_width, 0);
            // TODO this should be safe after layout, but I might want to add a no-panic default.
            let frame = Rect::new(shift, parent_size - shift);
            let suboutput = SubOutput::new(output, frame);

            Some(suboutput)
        } else {
            None
        };

        // This is removing one or both constraints to enable scrolling
        let mut over_output = match sub_output.as_mut() {
            Some(sub_output) => OverOutput::new(sub_output, new_sc),
            None => OverOutput::new(output, new_sc),
        };

        self.widget.render(theme, focused, &mut over_output);
    }

    fn anchor(&self) -> XY {
        // scroll nesting would probably affect that
        XY::ZERO
    }
}