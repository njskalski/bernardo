use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::output::{Metadata, Output};
use crate::io::over_output::OverOutput;
use crate::io::sub_output::SubOutput;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::{Scroll, ScrollDirection};
use crate::primitives::xy::XY;
use crate::unpack_or;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

// const DEFAULT_MARGIN_WIDTH: u16 = 4;

struct LayoutRes {
    margin_width: u16,
    size_of_new_output: XY,
}

pub struct WithScroll<W: Widget> {
    id: WID,
    child_widget: W,
    scroll: Scroll,
    line_no: bool,

    // TODO I guess that was for something but I forgot what was that
    fill_non_free_axis: bool,

    // margin, size of new output
    layout_res: Option<LayoutRes>,
}

impl<W: Widget> WithScroll<W> {
    pub const TYPENAME: &'static str = "with_scroll";
    pub const MIN_SIZE: XY = XY::new(3, 4);

    pub fn new(scroll_direction: ScrollDirection, widget: W) -> Self {
        let id = get_new_widget_id();
        Self {
            id,
            child_widget: widget,
            scroll: Scroll::new(scroll_direction),
            line_no: false,
            fill_non_free_axis: true,
            layout_res: None,
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

    pub fn internal_mut(&mut self) -> &mut W {
        &mut self.child_widget
    }

    pub fn internal(&self) -> &W {
        &self.child_widget
    }

    pub fn mutate_internal<F: FnOnce(W) -> W>(self, mutator: F) -> Self {
        Self {
            child_widget: mutator(self.child_widget),
            ..self
        }
    }

    fn line_count_margin_width_for_lower_right(&self) -> u16 {
        let lower_right = self.child_widget.full_size();
        let width = format!("{}", lower_right.y).len() as u16 + 2;
        width
    }

    fn render_line_no(&self, margin_width: u16, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let layout_res = unpack_or!(self.layout_res.as_ref(), (), "render before layout");
        #[cfg(test)]
        {
            output.emit_metadata(Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: Rect::from_zero(XY::new(margin_width, output.size().y)),
                focused,
            });

            assert!(false, "unimplemented");
        }

        debug_assert!(self.line_no);
        let start_idx = self.scroll.offset.y;

        let style = if focused {
            theme.ui.header
        } else {
            theme.ui.header.half()
        }.with_background(theme.default_text(focused).background);

        // let anchor_row = self.widget.anchor().y;

        // TODO narrow to visible rect
        for idx in 0..output.size().y {
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
        Self::TYPENAME
    }

    fn prelayout(&mut self) {
        // debug!("prelayout {}", self.typename());
        self.child_widget.prelayout();
    }

    fn full_size(&self) -> XY {
        let child_full_size = self.child_widget.full_size();
        let margin = if self.line_no { self.line_count_margin_width_for_lower_right() } else { 0 };

        let res = match self.scroll.direction {
            ScrollDirection::Horizontal => {
                XY::new(Self::MIN_SIZE.x + margin, child_full_size.y)
            }
            ScrollDirection::Vertical => {
                XY::new(child_full_size.x + margin, Self::MIN_SIZE.y)
            }
            ScrollDirection::Both => {
                Self::MIN_SIZE + XY::new(margin, 0)
            }
        };

        res
    }

    fn layout(&mut self, output_size: XY, visible_rect: Rect) {
        let child_full_size = self.child_widget.full_size();
        let margin_width = if self.line_no {
            self.line_count_margin_width_for_lower_right()
        } else {
            0 as u16
        };

        if !self.scroll.direction.free_x() {
            debug_assert!(child_full_size.x + margin_width as u16 <= output_size.x)
        }

        if !self.scroll.direction.free_y() {
            debug_assert!(child_full_size.y <= output_size.y)
        }

        let internal_output_size = XY::new(
            if self.scroll.direction.free_x() { child_full_size.x } else { output_size.x - margin_width },
            if self.scroll.direction.free_y() { child_full_size.y } else { output_size.y },
        );

        let mut internal_visible_rect = visible_rect;
        internal_visible_rect.pos = self.scroll.offset;
        internal_visible_rect.size -= XY::new(margin_width, 0);
        // TODO what if it's empty?

        self.child_widget.layout(internal_output_size, internal_visible_rect);

        self.scroll.follow_kite(internal_output_size,
                                self.child_widget.kite());

        self.layout_res = Some(LayoutRes {
            margin_width,
            size_of_new_output: internal_output_size,
        });
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!(target: "recursive_treat_views", "in scroll, passing {:?} to {:?}", &msg, &self.child_widget as &dyn Widget);
        // do NOT route the message down the tree again, that's the job of recursive_treat_views.
        // Pass it down through.
        Some(msg)
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        Some(&self.child_widget)
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(&mut self.child_widget)
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let layout_res = unpack_or!(self.layout_res.as_ref(), (), "render before layout");

        if layout_res.margin_width > 0 {
            self.render_line_no(layout_res.margin_width, theme, focused, output);
        }

        // This is narrowing the scope to make margin for line_no
        let mut sub_output: Option<SubOutput> = if self.line_no {
            let shift = XY::new(layout_res.margin_width, 0);
            // TODO this should be safe after layout, but I might want to add a no-panic default.
            let frame = Rect::new(shift, output.size() - shift);
            let suboutput = SubOutput::new(output, frame);

            Some(suboutput)
        } else {
            None
        };

        // This is removing one or both constraints to enable scrolling
        let mut over_output = match sub_output.as_mut() {
            Some(sub_output) => OverOutput::new(sub_output,
                                                layout_res.size_of_new_output,
                                                self.scroll.offset),
            None => OverOutput::new(output, layout_res.size_of_new_output, self.scroll.offset),
        };

        self.child_widget.render(theme, focused, &mut over_output);
    }

    fn kite(&self) -> XY {
        // scroll nesting would probably affect that
        XY::ZERO
    }
}