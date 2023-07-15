use std::cmp::{max, min};

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
use crate::widget::fill_policy::{DeterminedBy, SizePolicy};
use crate::widget::widget::{get_new_widget_id, WID, Widget};

// const DEFAULT_MARGIN_WIDTH: u16 = 4;

struct LayoutRes {
    margin_width: u16,
    parent_space_child_output_rect: Rect,
    child_space_visible_rect: Rect,
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

    // TODO It probably shouldn't be here, but then this acts as Layout anyways
    size_policy: SizePolicy,
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
            size_policy: SizePolicy::MATCH_LAYOUT,
        }
    }

    pub fn with_size_policy(self, size_policy: SizePolicy) -> Self {
        Self {
            size_policy,
            ..self
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

        for idx in output.visible_rect().pos.y..output.visible_rect().lower_right().y {
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

        let x = match (self.size_policy.x, self.scroll.direction.free_x()) {
            (DeterminedBy::Widget, true) => { child_full_size.x }
            (DeterminedBy::Widget, false) => {
                debug_assert!(child_full_size.x + margin_width <= output_size.x);
                min(child_full_size.x, output_size.x - margin_width)
            }
            (DeterminedBy::Layout, true) => {
                max(output_size.x - margin_width, child_full_size.x)
            }
            (DeterminedBy::Layout, false) => {
                debug_assert!(child_full_size.x + margin_width <= output_size.x);
                output_size.x - margin_width
            }
        };

        let y = match (self.size_policy.y, self.scroll.direction.free_y()) {
            (DeterminedBy::Widget, true) => { child_full_size.y }
            (DeterminedBy::Widget, false) => {
                debug_assert!(child_full_size.y <= output_size.y);
                min(child_full_size.y, output_size.y)
            }
            (DeterminedBy::Layout, true) => {
                max(output_size.y, child_full_size.y)
            }
            (DeterminedBy::Layout, false) => {
                debug_assert!(child_full_size.y + margin_width <= output_size.y);
                output_size.y - margin_width
            }
        };

        let parent_space_child_output_rect = Rect::new(XY::new(margin_width, 0), XY::new(x, y));

        // TODO what if it's empty?
        let mut child_space_visible_rect = visible_rect.intersect(parent_space_child_output_rect).unwrap();
        child_space_visible_rect = child_space_visible_rect.minus_shift(self.scroll.offset + XY::new(margin_width, 0)).unwrap();

        self.child_widget.layout(parent_space_child_output_rect.size, child_space_visible_rect);

        self.scroll.follow_kite(parent_space_child_output_rect.size,
                                self.child_widget.kite());

        debug_assert!(parent_space_child_output_rect.lower_right() == output_size,
                      "parent_space_child_output_rect.lower_right() = {} != {} = output_size, rect {}",
                      parent_space_child_output_rect.lower_right(), output_size, parent_space_child_output_rect,
        );

        self.layout_res = Some(LayoutRes {
            margin_width,
            parent_space_child_output_rect,
            child_space_visible_rect,
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

        debug_assert!((layout_res.margin_width > 0) == self.line_no);

        // This is narrowing the scope to make margin for line_no
        let mut sub_output: Option<SubOutput> = if layout_res.margin_width > 0 {
            debug_assert!(layout_res.parent_space_child_output_rect.pos != XY::ZERO);
            let suboutput = SubOutput::new(output, layout_res.parent_space_child_output_rect);
            Some(suboutput)
        } else {
            debug_assert!(layout_res.parent_space_child_output_rect.pos == XY::ZERO);
            None
        };

        // This is removing one or both constraints to enable scrolling
        let mut over_output = match sub_output.as_mut() {
            Some(sub_output) => OverOutput::new(sub_output,
                                                layout_res.parent_space_child_output_rect.size,
                                                self.scroll.offset),
            None => OverOutput::new(output, layout_res.parent_space_child_output_rect.size, self.scroll.offset),
        };

        self.child_widget.render(theme, focused, &mut over_output);
    }

    fn kite(&self) -> XY {
        // scroll nesting would probably affect that
        XY::ZERO
    }
}