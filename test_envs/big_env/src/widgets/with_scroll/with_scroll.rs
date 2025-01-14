use std::cmp::{max, min};

use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::io::over_output::OverOutput;
use crate::io::sub_output::SubOutput;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::{Scroll, ScrollDirection};
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::{DeterminedBy, SizePolicy};
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::{unpack_or_e, unpack_unit};

// const DEFAULT_MARGIN_WIDTH: u16 = 4;

struct LayoutRes {
    margin_width: u16,
    parent_space_child_output_rect: Rect,
    child_space_output_size: XY,
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

    // Used to inform a full_size when used outside greedy layout
    max_size: Option<XY>,
}

struct InternalOutputSize {
    child_size_in_its_output: XY,
    margin_width: u16,
}

impl<W: Widget> WithScroll<W> {
    pub const TYPENAME: &'static str = "with_scroll";

    pub const TYPENAME_FOR_MARGIN: &'static str = "with_scroll_margin";

    pub fn new(scroll_direction: ScrollDirection, widget: W) -> Self {
        let id = get_new_widget_id();
        Self {
            id,
            child_widget: widget,
            scroll: Scroll::new(scroll_direction),
            line_no: false,
            fill_non_free_axis: true,
            layout_res: None,
            max_size: None,
        }
    }

    pub fn with_max_size(self, max_size: XY) -> Self {
        Self {
            max_size: Some(max_size),
            ..self
        }
    }

    pub fn with_line_no(self) -> Self {
        Self { line_no: true, ..self }
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

    pub fn mutate_internal<F: FnOnce(W) -> W>(mut self, mutator: F) -> Self {
        Self {
            child_widget: mutator(self.child_widget),
            ..self
        }
    }

    fn render_line_no(&self, margin_width: u16, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let _layout_res = unpack_unit!(self.layout_res.as_ref(), "render before layout",);
        #[cfg(test)]
        {
            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: Self::TYPENAME_FOR_MARGIN.to_string(),
                rect: Rect::from_zero(XY::new(margin_width, output.size().y)),
                focused,
            });
        }

        debug_assert!(self.line_no);
        let start_idx = self.scroll.offset.y;

        let style = if focused { theme.ui.header } else { theme.ui.header.half() }.with_background(theme.default_text(focused).background);

        for idx in output.visible_rect().pos.y..output.visible_rect().lower_right().y {
            let line_no_base_0 = start_idx + idx;
            let item = format!("{} ", line_no_base_0 + 1);
            let num_digits_plus_one = item.len() as u16;
            let offset = if num_digits_plus_one <= margin_width {
                margin_width - num_digits_plus_one
            } else {
                error!("num_digits > margin_width, hardcoding safe fix");
                0
            };

            // let style = if line_no_base_0 == anchor_row {
            //     style.with_background(theme.ui.cursors.background)
            // } else { style };

            for px in 0..offset {
                output.print_at(XY::new(px, idx), style, " ")
            }

            output.print_at(XY::new(offset, idx), style, &item);
        }
    }

    fn get_margin_width_for_height(height: u16) -> u16 {
        format!("{}", height).width() as u16 + 2 // TODO logarithm? Never heard of it.
    }

    fn get_output_size_that_will_be_offered_to_child(&self, output_size: XY) -> InternalOutputSize {
        /*
        This is a little over-documented function, but I am tired and I make mistakes, so it's quicker to just dump my brain then investigate "what was I thinking" later.
         */

        // let's decide how much space will be offered to child first.
        let child_full_size = self.child_widget.full_size();
        let mut internal_output_size = child_full_size;

        if self.scroll.direction.free_y() {
            // we have infinite y, let's see what child widget would like to do.
            if self.child_widget.size_policy().y == DeterminedBy::Widget {
                debug!(
                    "y1 Widget {} decides height, it has infinite space. So it takes child_full_size.y = {}",
                    self.child_widget.typename(),
                    child_full_size.y
                );
                internal_output_size.y = child_full_size.y;
            } else {
                internal_output_size.y = max(child_full_size.y, output_size.y);
                debug!("y2 Widget {} relies on layout to decide height, it has infinite space and is at most {} tall. So it takes max(child_full_size.y = {}, output_size.y = {}) = {}",
                    self.child_widget.typename(),
                    child_full_size.y, child_full_size.y, output_size.y, internal_output_size.y);
            }
        } else {
            // we have at most output_size.y cells available
            if self.child_widget.size_policy().y == DeterminedBy::Widget {
                internal_output_size.y = min(child_full_size.y, output_size.y);
                debug!("y3 Widget {} decides height, it has FINITE space. It takes child_full_size.y = {}, layout height is = {}. It takes min() = {}",
                    self.child_widget.typename(),
                    child_full_size.y, output_size.y, internal_output_size.y);
            } else {
                debug!(
                    "y4 Widget {} relies on layout to decide height, layout is {} high.",
                    self.child_widget.typename(),
                    output_size.y
                );
                internal_output_size.y = output_size.y;
            }
        }

        // now that we know height y, we can see what's our final width.
        let (margin_width, max_output_width) = if self.line_no {
            let margin_width = Self::get_margin_width_for_height(internal_output_size.y);
            debug!(
                "having {} lines to count, I need {} width for the numbers.",
                internal_output_size.y, margin_width
            );

            if margin_width > output_size.x {
                error!(
                    "margin_width = {} > {} = output_size.x, this is a TODO.",
                    margin_width, output_size.x
                );
                (0, output_size.x)
            } else {
                (margin_width, output_size.x - margin_width)
            }
        } else {
            debug!("with no line numbers, I can use full width of {}", output_size.x);
            (0, output_size.x)
        };

        if self.scroll.direction.free_x() {
            if self.child_widget.size_policy().x == DeterminedBy::Widget {
                debug!(
                    "x1 Widget {} decides width, it has infinite space. So it takes child_full_size.x = {}",
                    self.child_widget.typename(),
                    child_full_size.x
                );
                internal_output_size.x = child_full_size.x;
            } else {
                internal_output_size.x = max(child_full_size.x, max_output_width);
                debug!("x2 Widget {} relies on layout to decide width, it has infinite space and is at most {} wide. So it takes max(child_full_size.x = {}, max_output_width = {}) = {}",
                    self.child_widget.typename(),
                    child_full_size.x, child_full_size.x, max_output_width, internal_output_size.x);
            }
        } else {
            // we have at most max_output_width cells available
            if self.child_widget.size_policy().x == DeterminedBy::Widget {
                internal_output_size.x = min(child_full_size.x, max_output_width);
                debug!("x3 Widget {} decides width, it has FINITE space. It takes child_full_size.x = {}, max_output_width is = {}. It takes min() = {}",
                    self.child_widget.typename(),
                    child_full_size.x, max_output_width, internal_output_size.x);
            } else {
                debug!(
                    "x4 Widget {} relies on layout to decide width, layout is {} wide.",
                    self.child_widget.typename(),
                    max_output_width
                );
                internal_output_size.x = max_output_width;
            }
        }

        InternalOutputSize {
            child_size_in_its_output: internal_output_size,
            margin_width,
        }
    }
}

impl<W: Widget> Widget for WithScroll<W> {
    fn id(&self) -> WID {
        self.id
    }
    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }
    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn prelayout(&mut self) {
        // debug!("prelayout {}", self.typename());
        self.child_widget.prelayout();
    }

    fn full_size(&self) -> XY {
        if let Some(max_size) = self.max_size {
            let full_size = self.internal().full_size();

            XY::new(min(max_size.x, full_size.x), min(max_size.y, full_size.y))
        } else {
            debug_assert!(
                false,
                "this shouldn't be even called, with scroll should be used only with greedy layout"
            );
            // this code should be dead, ignore it.
            XY::new(3, 3)
        }
    }

    fn size_policy(&self) -> SizePolicy {
        // this has to be MATCH_LAYOUT in order for full_size to not be called.
        // If I want the scrolled things to not be completely greedy, I'd need to change the contract.
        // But I don't think it makes sense, it's super easy to create artificial constraint.

        SizePolicy::MATCH_LAYOUT
    }

    fn layout(&mut self, screenspace: Screenspace) {
        //TODO write more tests and redo the code (perhaps)
        self.layout_res = None; // erasing old layout_res

        let child_output = self.get_output_size_that_will_be_offered_to_child(screenspace.output_size());

        if child_output.child_size_in_its_output.x == 0 || child_output.child_size_in_its_output.y == 0 {
            error!(
                "child output is degenerated ({}), will not lay out further.",
                child_output.child_size_in_its_output
            );
            return;
        }

        if child_output.margin_width > screenspace.output_size().x {
            warn!(
                "margin ({}) is wider than visible rect ({}), will not lay out further.",
                child_output.margin_width,
                screenspace.visible_rect().size.x
            );
            return;
        }

        let child_visible_rect_pos_in_parent_space = XY::new(child_output.margin_width, 0);

        // this is the maximum space (constraint) that we *can offer* to the child, so the output of parent
        // - the margin.
        let parent_space_maximum_child_output_rect = /* output part that can be offered to child*/ Rect::new(child_visible_rect_pos_in_parent_space, screenspace.output_size() - child_visible_rect_pos_in_parent_space);

        // this is tricky part: I take "child_size_in_its_output" which is "how much space child will 'see'
        // as in it's output", but we move it to parent space. This has no logical meaning other
        // than I want it in parent space, to intersect it with "parent_space_maximum_child_output_rect"
        // to get the final constraint.
        let parent_space_child_output_rect_uncut = Rect::new(child_visible_rect_pos_in_parent_space, child_output.child_size_in_its_output);

        let parent_space_child_output_rect = parent_space_maximum_child_output_rect
            .intersect(parent_space_child_output_rect_uncut)
            .unwrap(); //TODO prove this can't go wrong.

        let child_visible_rect_in_parent_space: Rect = match screenspace.visible_rect().intersect(parent_space_child_output_rect) {
            Some(intersection) => {
                debug!(
                    "in parent space, visible rect is {}, so with {} margin, child has {}",
                    screenspace.visible_rect(),
                    child_output.margin_width,
                    intersection
                );
                intersection
            }
            None => {
                debug!("intersection between visible rect {} and space we can offer child after substracting {} margin was empty, so we do not layout further.", screenspace.visible_rect(), child_output.margin_width);
                return;
            }
        };

        let mut child_visible_rect_in_child_space: Rect =
            match child_visible_rect_in_parent_space.minus_shift(child_visible_rect_pos_in_parent_space) {
                Some(s) => s,
                None => {
                    error!("impossible: failed to unwrap minus shift of some other shift.");
                    return;
                }
            };

        debug_assert!(child_visible_rect_in_child_space.size == child_visible_rect_in_parent_space.size);

        // This is where scroll actually follows the widget.
        // I need to update the scroll offset first to use it in next step.
        debug!(
            "following kite at {} with output_size() = {} vis_rect = {}",
            self.child_widget.kite(),
            screenspace.output_size(),
            screenspace.visible_rect()
        );

        self.scroll.follow_kite(
            child_visible_rect_in_child_space.size,
            child_output.child_size_in_its_output,
            self.child_widget.kite(),
        );

        // this line came about via trial-and-error in tests. That probably invalidates
        // a lot of code above, but I am on vacation and I have too little screen here to
        // redo entire code.
        child_visible_rect_in_child_space.pos += self.scroll.offset;

        let child_screenspace = Screenspace::new(child_output.child_size_in_its_output, child_visible_rect_in_child_space);

        debug_assert!(
            self.child_widget.kite().x < child_screenspace.visible_rect().max_x(),
            "kite {} child_screenspace.visible_rect().lower_right() = {}",
            self.child_widget.kite(),
            child_screenspace.visible_rect().lower_right()
        );
        debug_assert!(
            self.child_widget.kite().y < child_screenspace.visible_rect().max_y(),
            "kite {} child_screenspace.visible_rect().lower_right() = {}",
            self.child_widget.kite(),
            child_screenspace.visible_rect().lower_right()
        );

        self.child_widget.layout(child_screenspace);

        self.layout_res = Some(LayoutRes {
            margin_width: child_output.margin_width,
            parent_space_child_output_rect: parent_space_maximum_child_output_rect,
            child_space_output_size: child_output.child_size_in_its_output,
            child_space_visible_rect: child_visible_rect_in_child_space,
        });
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        self.internal().on_input(input_event)
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("in scroll, NOT passing {:?} to {:?}", &msg, &self.child_widget as &dyn Widget);
        // do NOT route the message down the tree again, that's the job of act_on() method.
        // update bubbles results UP
        Some(msg)
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        Some(&self.child_widget)
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(&mut self.child_widget)
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            output.emit_metadata(crate::io::output::Metadata {
                id: self.id,
                typename: self.typename().to_string(),
                rect: Rect::from_zero(output.size()),
                focused,
            });
        }

        let layout_res = unpack_unit!(self.layout_res.as_ref(), "render before layout");

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
            Some(sub_output) => OverOutput::new(sub_output, layout_res.child_space_output_size, self.scroll.offset),
            None => OverOutput::new(output, layout_res.child_space_output_size, self.scroll.offset),
        };

        debug_assert!(over_output.visible_rect().lower_right().x > self.child_widget.kite().x);
        debug_assert!(over_output.visible_rect().lower_right().y > self.child_widget.kite().y);

        self.child_widget.render(theme, focused, &mut over_output);
    }

    fn kite(&self) -> XY {
        let child_kite = self.child_widget.kite();

        let lr = unpack_or_e!(self.layout_res.as_ref(), XY::ZERO, "failed to get kite before layout");

        debug_assert!(child_kite >= self.scroll.offset);

        child_kite - self.scroll.offset + XY::new(lr.margin_width, 0)
    }
}
