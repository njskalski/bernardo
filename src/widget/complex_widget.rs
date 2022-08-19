use log::{error, warn};

use crate::{Output, SizeConstraint, Theme, Widget};
use crate::experiments::focus_group::{FocusGraph, FocusUpdate};
use crate::experiments::from_geometry::from_geometry;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::sub_output::SubOutput;
use crate::layout::layout::{Layout, WidgetWithRect};
use crate::primitives::helpers::fill_output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::WID;

// here one could merge focus_group.focused with ds.focused, but not it's not important.

pub struct DisplayState<S: Widget> {
    focused: SubwidgetPointer<S>,
    wwrs: Vec<WidgetWithRect<S>>,
    focus_group: FocusGraph<SubwidgetPointer<S>>,
}

pub trait ComplexWidget: Widget + Sized {
    fn internal_layout(&self, max_size: XY) -> Box<dyn Layout<Self>>;
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

    fn complex_layout(&mut self, sc: SizeConstraint) -> XY {
        let xy = sc.as_finite().unwrap_or_else(|| {
            warn!("using complex_layout on infinite SizeConstraint is not supported, will limit itself to visible hint");
            sc.visible_hint().size
        });

        let layout = self.internal_layout(xy);
        let wwrs = layout.layout(self, xy);

        let widgets_and_positions: Vec<(WID, SubwidgetPointer<Self>, Rect)> = wwrs.iter().map(|w| {
            let rect = w.rect().clone();
            let wid = w.widget().get(self).id();
            (wid, w.widget().clone(), rect)
        }).collect();

        let focused = self.get_display_state_op()
            .as_ref()
            .map(|s| s.focused.clone())
            .unwrap_or(self.get_default_focused());

        let selected = focused.get(self).id();

        let focus_group = from_geometry::<>(&widgets_and_positions, selected, xy);

        let new_state = DisplayState {
            focused,
            wwrs,
            focus_group,
        };

        self.set_display_state(new_state);

        xy
    }

    fn complex_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.ui.non_focused.background, output);

        let mut focused_drawn = false;

        match self.get_display_state_op() {
            None => error!("failed rendering save_file_dialog without cached_sizes"),
            Some(ds) => {
                let focused_subwidget = ds.focused.get(self);

                for wwr in &ds.wwrs {
                    let sub_output = &mut SubOutput::new(output, *wwr.rect());
                    let widget = wwr.widget().get(self);
                    let subwidget_focused = focused && widget.id() == focused_subwidget.id();
                    widget.render(theme,
                                  subwidget_focused,
                                  sub_output);

                    focused_drawn |= subwidget_focused;
                }
            }
        }

        if !focused_drawn {
            error!("a focused widget is not drawn in {} #{}!", self.typename(), self.id())
        }
    }

    fn set_focused(&mut self, subwidget_pointer: SubwidgetPointer<Self>) {
        if let Some(ds) = self.get_display_state_mut_op() {
            ds.focused = subwidget_pointer;
        } else {
            error!("failed setting focused before layout. Use get_default_focused instead?");
        }
    }

    fn complex_get_focused(&self) -> Option<&dyn Widget> {
        if self.get_display_state_op().is_none() {
            error!("requested complex_get_focused before layout");
        }

        self.get_display_state_op().as_ref().map(|ds| ds.focused.get(self))
    }

    fn complex_get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        if self.get_display_state_op().is_none() {
            error!("requested complex_get_focused_mut before layout");
        }

        let focused_ptr =

            self.get_display_state_op().as_ref().map(|ds| {
                ds.focused.clone()
            });

        focused_ptr.map(|p| p.get_mut(self))
    }
}
