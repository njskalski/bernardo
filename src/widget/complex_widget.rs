use log::{error, warn};

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::sub_output::SubOutput;
use crate::layout::layout::{Layout, WidgetWithRect};
use crate::primitives::helpers::fill_output;
use crate::primitives::xy::XY;
use crate::widget::widget::WID;

pub struct DisplayState<S: Widget> {
    focused: SubwidgetPointer<S>,
    wwrs: Vec<WidgetWithRect<S>>,
}

// S is for "Self"
pub trait ComplexWidget: Widget {
    fn internal_layout(&self, max_size: XY) -> Box<dyn Layout<Self>> where Self: Sized;
    fn get_default_focused(&self) -> SubwidgetPointer<Self> where Self: Sized;

    fn set_display_state(&mut self, ds: DisplayState<Self>) where Self: Sized;
    fn get_display_state_op(&self) -> &Option<DisplayState<Self>> where Self: Sized;

    fn complex_layout(&mut self, sc: SizeConstraint) -> XY where Self: Sized {
        let xy = sc.as_finite().unwrap_or_else(|| {
            warn!("using complex_layout on infinite SizeConstraint is not supported, will limit itself to visible hint");
            sc.visible_hint().size
        });

        let layout = self.internal_layout(xy);
        let wwrs = layout.layout(self, xy);

        let focused = self.get_display_state_op()
            .as_ref()
            .map(|s| s.focused.clone())
            .unwrap_or(self.get_default_focused());

        let new_state = DisplayState {
            focused,
            wwrs,
        };

        self.set_display_state(new_state);

        xy
    }

    fn complex_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) where Self: Sized {
        fill_output(theme.ui.non_focused.background, output);

        match self.get_display_state_op() {
            None => error!("failed rendering save_file_dialog without cached_sizes"),
            Some(ds) => {
                let focused_subwidget = ds.focused.get(self);

                for wwr in &ds.wwrs {
                    let sub_output = &mut SubOutput::new(output, *wwr.rect());
                    let widget = wwr.widget().get(self);
                    widget.render(theme,
                                  focused && widget.id() == focused_subwidget.id(),
                                  sub_output);
                }
            }
        }
    }

    fn set_focused(&mut self, subwidget_pointer: SubwidgetPointer<Self>) where Self: Sized {
        if let Some(ds) = self.get_display_state_op() {
            let new_ds = DisplayState {
                focused: subwidget_pointer,
                wwrs: ds.wwrs.clone(),
            };

            self.set_display_state(new_ds);
        } else {
            error!("failed setting focused before layout. Use get_default_focused instead?");
        }
    }

    fn complex_get_focused(&self) -> Option<&dyn Widget> where Self: Sized {
        if self.get_display_state_op().is_none() {
            error!("requested complex_get_focused before layout");
        }

        self.get_display_state_op().as_ref().map(|ds| ds.focused.get(self))
    }

    fn complex_get_focused_mut(&mut self) -> Option<&mut dyn Widget> where Self: Sized {
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
