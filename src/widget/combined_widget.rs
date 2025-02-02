use log::{debug, error};

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::layout::{Layout, LayoutResult};
use crate::primitives::helpers::fill_output;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;

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

    fn get_subwidgets_for_input(&self) -> impl Iterator<Item=SubwidgetPointer<Self>>;
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

    // fn combined_act_on(&mut self, input_event: InputEvent) -> (bool, Option<Box<dyn AnyMsg>>) {
    //     let desc = self.desc();
    //
    //     debug!(target: "act_on", "combined 1: {} consumed considering {:?}", &desc, &input_event);
    //     let children_ptrs: Vec<SubwidgetPointer<Self>> = self.get_subwidgets_for_input().collect();
    //     for child_ptr in children_ptrs {
    //         let child = child_ptr.get_mut(self);
    //         let child_desc = child.desc();
    //
    //         debug!(target: "act_on", "combined 2: {} offering {:?} to {}", &desc, &input_event, &child_desc);
    //         let (consumed, message_to_child_self_op) = child.act_on(input_event);
    //         debug!(target: "act_on", "combined 3: {}'s child {} consumed ({}), offered message_to_self {:?}", &desc, &child_desc, consumed, &message_to_child_self_op);
    //
    //         if consumed {
    //             if let Some(message_to_child_self) = message_to_child_self_op {
    //                 let message_to_self_op = child.update(message_to_child_self);
    //                 debug!(target: "act_on", "combined 4: {}'s child {} produced {:?}", &desc, &child_desc, &message_to_self_op);
    //
    //                 if let Some(message_to_self) = message_to_self_op {
    //                     let message_to_parent = self.update(message_to_self);
    //                     debug!(target: "act_on", "combined 5: {} produced message_to_parent {:?} ", &desc, &message_to_parent);
    //                     return (true, message_to_parent);
    //                 } else {
    //                     debug!(target: "act_on", "combined 5: {} produced NO message_to_parent", &desc);
    //                     return (true, None);
    //                 }
    //             } else {
    //                 return (true, None);
    //             }
    //         } else {
    //             continue;
    //         }
    //     }
    //
    //     (false, None)
    // }
}
