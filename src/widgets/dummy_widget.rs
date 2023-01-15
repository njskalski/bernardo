// use log::error;
//
// use crate::config::theme::Theme;
// use crate::io::input_event::InputEvent;
// use crate::io::output::{Metadata, Output};
// use crate::primitives::rect::Rect;
// use crate::primitives::size_constraint::SizeConstraint;
// use crate::primitives::xy::XY;
// use crate::unpack_or;
// use crate::widget::any_msg::AnyMsg;
// use crate::widget::fill_policy::FillPolicy;
// use crate::widget::widget::{get_new_widget_id, WID, Widget};
//
// struct DummyWidget {
//     wid: WID,
//     fill_policy: FillPolicy,
//     size: XY,
//     text_op: Option<String>,
// }
//
// impl DummyWidget {
//     pub const TYPENAME: &'static str = "dummy_widget";
//
//     pub fn new(size: XY) -> Self {
//         DummyWidget {
//             wid: get_new_widget_id(),
//             fill_policy: FillPolicy::CONSTRAINED,
//             size,
//             text_op: None,
//         }
//     }
//
//     pub fn with_fill_policy(self, fill_policy: FillPolicy) -> Self {
//         Self {
//             fill_policy,
//             ..self
//         }
//     }
//
//     pub fn with_text(self, text: String) -> Self {
//         Self {
//             text_op: Some(text),
//             ..self
//         }
//     }
// }
//
//
// impl Widget for DummyWidget {
//     fn id(&self) -> WID {
//         self.wid
//     }
//
//     fn typename(&self) -> &'static str {
//         Self::TYPENAME
//     }
//
//     fn min_size(&self) -> XY {
//         self.size
//     }
//
//     fn layout(&mut self, sc: SizeConstraint) -> XY {
//         self.fill_policy.get_size_from_constraints()
//
//         let size = sc.as_finite().unwrap_or_else(|| {
//             error!("non-simple size constratint on expanding widget, using min size");
//             self.min_size()
//         });
//
//         let mut x = 0;
//         if size.x >= Self::NO_EDIT_TEXT.len() as u16 {
//             x = (size.x - Self::NO_EDIT_TEXT.len() as u16) / 2;
//         };
//
//         let y = size.y / 2;
//
//         self.text_pos = XY::new(x, y);
//         self.last_size = Some(size);
//         size
//     }
//
//     fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> { None }
//
//     fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
//         None
//     }
//
//     fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
//         #[cfg(test)]
//         {
//             let size = unpack_or!(self.last_size, (), "render before layout");
//             output.emit_metadata(
//                 Metadata {
//                     id: self.wid,
//                     typename: self.typename().to_string(),
//                     rect: Rect::from_zero(size),
//                     focused,
//                 }
//             );
//         }
//
//         // fill_background(theme.default_background(focused), output);
//
//         output.print_at(self.text_pos,
//                         theme.default_text(focused),
//                         Self::NO_EDIT_TEXT,
//         );
//     }
// }