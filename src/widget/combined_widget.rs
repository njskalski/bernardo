use log::{debug, error};

use crate::config::theme::Theme;
use crate::experiments::focus_group::{FocusGraph, FocusUpdate};
use crate::experiments::from_geometry::from_geometry;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::helpers::fill_output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{Widget, WID};

/*
A combine widget is a widget that merges more than one widget using standard layout mechanisms,
but treats them all as one, as opposed to complex widget, which treats them individually.

There is no focus transfers within a CombinedWidgets, all subwidgets are focused or not *together as one*.
 */

pub trait CombinedWidget: Widget + Sized {
    fn internal_prelayout(&mut self) {}

    fn combined_prelayout(&mut self) {
        self.internal_prelayout();

        let layout = self.get_layout();
        layout.prelayout(self);
    }

    fn get_layout(&self) -> Box<dyn Layout<Self>>;

    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.default_text(focused).background, output);
    }

    fn save_layout_res(&mut self, result: LayoutResult<Self>);
    fn get_layout_res(&self) -> Option<&LayoutResult<Self>>;

    fn combined_layout(&mut self, screenspace: Screenspace) {
        let layout = self.get_layout();
        let layout_res = layout.layout(self, screenspace);
        self.save_layout_res(layout_res);
    }

    fn combined_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.ui.non_focused.background, output);

        let visible_rect = output.visible_rect();

        if let Some(layout_res) = self.get_layout_res() {
            let self_id = self.id();

            for wwr in layout_res.wwrs.iter() {
                let widget = wwr.widget().get(self);
                let _child_widget_desc = format!("{}", widget);

                if visible_rect.intersect(wwr.rect()).is_none() {
                    debug!(
                        "culling child widget {} of {}, no intersection between visible rect {} and wwr.rect {}",
                        widget,
                        self.typename(),
                        visible_rect,
                        wwr.rect()
                    );
                    continue;
                }

                let sub_output = &mut SubOutput::new(output, wwr.rect());

                if widget.id() != self_id {
                    widget.render(theme, focused, sub_output);
                } else {
                    self.internal_render(theme, focused, sub_output);
                }
            }
        } else {
            error!("render {} before layout", self.typename())
        }
    }
}
