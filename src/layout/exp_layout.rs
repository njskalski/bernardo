// use crate::layout::layout::{Layout};
// use crate::primitives::xy::XY;
// use crate::primitives::rect::Rect;
// use crate::widget::widget::{BaseWidget, WID};
// use crate::experiments::focus_group::FocusUpdate;
// use std::slice::Iter;
//
// pub type WidgetGetter<T : BaseWidget> = Box<dyn Fn(&mut T) -> &mut dyn BaseWidget>;
//
// pub struct ExpLayout<T : BaseWidget> {
//     widget_getter : WidgetGetter<T>,
// }
//
// impl <T: BaseWidget> ExpLayout<T> {
//     pub fn new(getter : WidgetGetter<T>) -> Self {
//         ExpLayout{
//             widget_getter: getter
//         }
//     }
//
// }
//
// impl <T: BaseWidget> Layout for ExpLayout<T> {
//     fn get_focused(&self) -> usize {
//         let x = (self.widget_getter)();
//         x
//     }
//
//     fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
//         false
//     }
//
//     fn get_rect(&self, output_size: XY, widget_id: WID) -> Option<Rect> {
//         if self.widget_id == widget_id {
//             Some(Rect::new(XY::new(0,0), output_size))
//         } else {
//             None
//         }
//     }
//
//     fn is_leaf(&self) -> bool {
//         true
//     }
//
//     fn has_id(&self, widget_id: WID) -> bool {
//         self.widget_id == widget_id
//     }
//
//     fn get_ids(&self) -> Vec<WID> {
//         vec![self.widget_id]
//     }
// }
