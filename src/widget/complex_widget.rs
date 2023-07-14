use log::error;

use crate::config::theme::Theme;
use crate::experiments::focus_group::{FocusGraph, FocusUpdate};
use crate::experiments::from_geometry::from_geometry;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::layout::Layout;
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::helpers::fill_output;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

// here one could merge focus_group.focused with ds.focused, but not it's not important.

/*
This widget got super complicated because I allowed widgets that are not "transparent", meaning they
have their own render and not just layout (CompletionWidget). This led to conditions in render,
layout, get_focused{ and mut} . Not sure if I want to keep it this way, or clean it up later.
 */

// TODO add "for size" here, I need it to do right "emit_*"
pub struct DisplayState<S: Widget> {
    pub focused: SubwidgetPointer<S>,
    pub wwrs: Vec<WidgetWithRect<S>>,
    pub focus_group: FocusGraph<SubwidgetPointer<S>>,
    pub total_size: XY,
}

pub trait ComplexWidget: Widget + Sized {
    fn internal_prelayout(&mut self) {}

    fn complex_prelayout(&mut self) {
        self.internal_prelayout();

        let layout = self.get_layout();
        layout.prelayout(self);
    }

    /*
    produces cloneable layout func tree
     */
    fn get_layout(&self) -> Box<dyn Layout<Self>>;

    /*
    because using ComplexWidget helper requires routing calling complex_render from widget's render,
    we require internal_render on widget's self, to avoid infinite recursion
     */
    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.default_text(focused).background, output);
    }

    // Called when we set initial "focus" within a complex widget and then as a fallback, whenever
    // we fail to retrieve focus from view state. Right now it's implemented *within the getters*
    // but TODO in a while I might bubble it up
    fn get_default_focused(&self) -> SubwidgetPointer<Self>;

    fn set_display_state(&mut self, display_state: DisplayState<Self>);
    fn get_display_state_op(&self) -> Option<&DisplayState<Self>>;
    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>>;

    fn will_accept_focus_update(&self, focus_update: FocusUpdate) -> bool {
        if let Some(ds) = self.get_display_state_op() {
            ds.focus_group.can_update_focus(focus_update)
        } else {
            error!("requested will_accept_focus_update before layout");
            false
        }
    }

    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
        if let Some(ds) = self.get_display_state_mut_op() {
            if ds.focus_group.update_focus(focus_update) {
                let subwidget_ptr = ds.focus_group.get_focused();
                ds.focused = subwidget_ptr;

                true
            } else {
                false
            }
        } else {
            error!("failed updating focus - display state not found");
            false
        }
    }

    /*
    This function automatically assumes you want to consume entire visible space with layout.
    If you want something else, override it. But remember to set display_cache or render will fail.
     */
    fn complex_layout(&mut self, output_size: XY, visible_rect: Rect) {
        let layout = self.get_layout();
        let layout_res = layout.layout(self, SizeConstraint::simple(output_size));

        for wwr in layout_res.wwrs.iter() {
            debug_assert!(output_size >= wwr.rect().lower_right());
            debug_assert!(layout_res.total_size >= wwr.rect().lower_right(), "total_size = {}, rect = {}", layout_res.total_size, wwr.rect());
        }

        let widgets_and_positions: Vec<(WID, SubwidgetPointer<Self>, Rect)> = layout_res.wwrs.iter().filter(
            |wwr| wwr.focusable()
        ).map(|w| {
            let rect = w.rect().clone();
            let wid = w.widget().get(self).id();
            (wid, w.widget().clone(), rect)
        }).collect();

        let focused = self.get_display_state_op()
            .as_ref()
            .map(|s| s.focused.clone())
            .unwrap_or(self.get_default_focused());

        let selected = focused.get(self).id();

        let focus_group = from_geometry::<>(&widgets_and_positions, selected, layout_res.total_size);

        let new_state = DisplayState {
            focused,
            wwrs: layout_res.wwrs,
            focus_group,
            total_size: layout_res.total_size,
        };

        self.set_display_state(new_state);

        layout_res.total_size
    }

    fn complex_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.ui.non_focused.background, output);

        match self.get_display_state_op() {
            None => error!("failed rendering {} without cached_sizes", self.typename()),
            Some(ds) => {
                let mut my_focused_drawn = false;
                let self_id = self.id();

                let focused_subwidget = ds.focused.get(self);
                let focused_desc = format!("{:?}", focused_subwidget);

                for wwr in &ds.wwrs {
                    let sub_output = &mut SubOutput::new(output, *wwr.rect());
                    let widget = wwr.widget().get(self);
                    let subwidget_focused = focused && widget.id() == focused_subwidget.id();

                    if widget.id() != self_id {
                        widget.render(theme,
                                      subwidget_focused,
                                      sub_output);
                    } else {
                        self.internal_render(theme, subwidget_focused, sub_output);
                    }
                    my_focused_drawn |= widget.id() == focused_subwidget.id();
                }

                if !my_focused_drawn {
                    error!("a focused widget {} is not drawn in {} #{}!", focused_desc, self.typename(), self.id())
                }
            }
        }
    }

    fn set_focused(&mut self, subwidget_pointer: SubwidgetPointer<Self>) {
        if let Some(ds) = self.get_display_state_mut_op() {
            ds.focused = subwidget_pointer;
        } else {
            error!("failed setting focused before layout on {}. Use get_default_focused instead?", self.typename());
        }
    }

    fn complex_get_focused(&self) -> Option<&dyn Widget> {
        if self.get_display_state_op().is_none() {
            error!("requested complex_get_focused before layout");
        }

        self.get_display_state_op().as_ref().map(|ds| {
            let w = ds.focused.get(self);
            if w.id() == self.id() {
                None
            } else {
                Some(w)
            }
        }).flatten()
    }

    fn complex_get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        if self.get_display_state_op().is_none() {
            error!("requested complex_get_focused_mut before layout");
        }

        let focused_ptr =

            self.get_display_state_op().as_ref().map(|ds| {
                ds.focused.clone()
            });

        focused_ptr.map(|p| {
            let self_id = self.id();
            let w = p.get_mut(self);
            if w.id() == self_id {
                None
            } else {
                Some(w)
            }
        }).flatten()
    }
}
