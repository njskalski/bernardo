// use crate::layout::layout::Layout;
// use crate::primitives::rect::Rect;
// use crate::experiments::focus_group::{FocusUpdate, FocusGroup};
// use crate::primitives::xy::XY;
// use crate::widget::widget::{WID, Widget};
// use std::collections::HashMap;
// use crate::experiments::from_geometry::from_geometry;
//
// pub struct FixedItem<W: Widget> {
//     pub layout : Box<dyn Layout<W>>,
//     pub rect : Rect
// }
//
// impl <W: Widget> FixedItem<W> {
//     pub fn new(layout : Box<dyn Layout<W>>, rect : Rect) -> Self {
//         FixedItem {
//             layout,
//             rect,
//         }
//     }
// }
//
// pub struct FixedLayout<W: Widget>  {
//     size : XY,
//     items : Vec<FixedItem<W>>,
//     focus : usize,
//     focus_group : Box<dyn FocusGroup>,
// }
//
// impl <W: Widget>  FixedLayout<W> {
//     pub fn new(size : XY, items : Vec<FixedItem<W>>) -> Self {
//
//         let all_items : Vec<(WID, Option<Rect>)> = items.iter()
//             .flat_map(|f| f.layout.get_all(size)).collect();
//
//         let fg = from_geometry(&all_items, Some(size));
//
//         FixedLayout {
//             size,
//             items,
//             focus : 0,
//             focus_group : Box::new(fg),
//         }
//     }
// }
//
// impl <W: Widget> Layout<W> for FixedLayout<W> {
//     fn get_focused<'a>(&self, parent: &'a W) -> &'a dyn Widget {
//         self.items[self.focus].layout.get_focused(parent)
//     }
//
//     fn get_focused_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget {
//         self.items[self.focus].layout.get_focused_mut(parent)
//     }
//
//     // fn get_focused(&self) -> usize {
//     //     self.items[self.focus].layout.get_focused()
//     // }
//
//     fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
//         self.focus_group.update_focus(focus_update)
//     }
//
//     fn get_rect(&self, output_size: XY, widget_id: WID) -> Option<Rect> {
//         match self.items.iter().find(|item| item.layout.has_id(widget_id)) {
//             None => None,
//             Some(right_item) => match right_item.layout.get_rect(right_item.rect.size, widget_id) {
//                 None => None,
//                 Some(mut rect) => {
//                     rect.pos = right_item.rect.pos;
//                     if rect.max_xy() <= output_size {
//                         Some(rect)
//                     } else {
//                         None
//                     }
//                 }
//             }
//         }
//     }
//
//     fn is_leaf(&self) -> bool {
//         false
//     }
//
//     fn has_id(&self, widget_id: WID) -> bool {
//         for fi in self.items.iter() {
//             if fi.layout.has_id(widget_id) {
//                 return true;
//             }
//         }
//         false
//     }
//
//     fn get_ids(&self) -> Vec<WID> {
//         self.items.iter().flat_map(|f| f.layout.get_ids()).collect()
//     }
// }
