// use core::option::Option;
// use std::alloc::Layout;
// use std::fmt::{Debug, Formatter};
// use std::rc::Rc;
//
// use log::{debug, error, warn};
// use unicode_width::UnicodeWidthStr;
//
// use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Theme, Widget, ZERO};
// use crate::io::keys::Key;
// use crate::layout::dummy_layout::DummyLayout;
// use crate::layout::layout::WidgetIdRect;
// use crate::primitives::border::BorderStyle;
// use crate::primitives::xy::XY;
// use crate::widget::any_msg::AsAny;
// use crate::widget::widget::{get_new_widget_id, WID, WidgetAction};
// use crate::widgets::button::ButtonWidget;
// use crate::widgets::generic_dialog::generic_dialog_msg::GenericDialogMsg;
//
// pub type KeyToMsg = (fn(&Key) -> Option<Box<dyn AnyMsg>>);
//
// const DEFAULT_INTERVAL: u16 = 2;
//
// pub struct DialogOption {
//     label: Rc<String>,
//     msg: Box<dyn AnyMsg>,
// }
//
// impl DialogOption {
//     pub fn new(label: Rc<String>, msg: Box<dyn AnyMsg>) -> Self {
//         Self {
//             label,
//             msg,
//         }
//     }
// }
//
// pub struct GenericDialog {
//     wid: WID,
//     text: Rc<String>,
//
//     with_border: Option<&'static BorderStyle>,
//
//     buttons: Vec<ButtonWidget<Rc<String>>>,
//     options: Vec<DialogOption>,
//     keystroke: Option<KeyToMsg>,
//
//     selected: usize,
//     on_cancel: WidgetAction<Self>,
// }
//
// impl GenericDialog {
//     pub fn new(text: Rc<String>, cancel: DialogOption, on_cancel: WidgetAction<Self>) -> Self {
//         let button: ButtonWidget<Rc<String>> = ButtonWidget::new(cancel.label.clone())
//             .with_on_hit(|_| GenericDialogMsg::Cancel.someboxed());
//
//         Self {
//             wid: get_new_widget_id(),
//             text,
//             with_border: None,
//             buttons: vec![button],
//             options: vec![cancel],
//             keystroke: None,
//             selected: 0,
//             on_cancel,
//         }
//     }
//
//     pub fn with_option(self, option: DialogOption) -> Self {
//         let mut options = self.options;
//         options.push(option);
//         Self {
//             options,
//             ..self
//         }
//     }
//
//     pub fn add_option(&mut self, option: DialogOption) {
//         let label = option.label.clone();
//
//         let idx = self.options.len();
//         self.options.push(option);
//         self.buttons.push(ButtonWidget::new(label)
//             .with_on_hit(|_| GenericDialogMsg::Hit(idx).someboxed())
//         );
//     }
//
//     pub fn with_border(self, border_style: &'static BorderStyle) -> Self {
//         Self {
//             with_border: Some(border_style),
//             ..self
//         }
//     }
//
//     pub fn text_size(&self) -> XY {
//         let mut size = ZERO;
//         for (idx, line) in self.text.lines().enumerate() {
//             size.x = size.x.max(line.width_cjk() as u16); // TODO
//             size.y = idx as u16;
//         }
//
//         size
//     }
//
//     pub fn get_total_options_width(&self, interval: u16) -> u16 {
//         let mut result: usize = 0;
//         for (idx, item) in self.options.iter().enumerate() {
//             result += item.label.width_cjk();
//             if idx + 1 < self.options.len() {
//                 result += interval as usize;
//             }
//         }
//         if result > u16::MAX as usize {
//             error!("absourdly long options_width, returning u16::MAX");
//             u16::MAX
//         } else {
//             result as u16
//         }
//     }
//
//     fn internal_layout(&self, size: XY) -> Vec<WidgetIdRect> {
//         let text = self.text_size();
//
//         vec![]
//     }
// }
//
// impl Widget for GenericDialog {
//     fn id(&self) -> WID {
//         self.wid
//     }
//
//     fn typename(&self) -> &'static str {
//         "GenericDialog"
//     }
//
//     fn min_size(&self) -> XY {
//         let mut total_size = self.text_size();
//
//         if !self.options.is_empty() {
//             let op_widths = self.get_total_options_width(DEFAULT_INTERVAL);
//
//             total_size.y += 2;
//             if total_size.x < op_widths {
//                 total_size.x = op_widths;
//             }
//         }
//
//         total_size + if self.with_border.is_some() { XY::new(2, 2) } else { ZERO }
//     }
//
//     fn layout(&mut self, sc: SizeConstraint) -> XY {
//         self.min_size()
//     }
//
//     fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
//         return match input_event {
//             InputEvent::KeyInput(key) => {
//                 match key.keycode {
//                     Keycode::Esc => GenericDialogMsg::Cancel.someboxed(),
//                     Keycode::ArrowLeft => GenericDialogMsg::Left.someboxed(),
//                     Keycode::ArrowRight => GenericDialogMsg::Right.someboxed(),
//                     _ => None,
//                 }
//             },
//             _ => None,
//         }
//     }
//
//     fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
//         debug!("save_file_dialog.update {:?}", msg);
//
//         let our_msg = msg.as_msg::<GenericDialogMsg>();
//         if our_msg.is_none() {
//             warn!("expecetd SaveFileDialogMsg, got {:?}", msg);
//             return None;
//         }
//
//         return match our_msg.unwrap() {
//             GenericDialogMsg::Cancel => (self.on_cancel)(self),
//             _ => None,
//         }
//     }
//
//     fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
//         todo!()
//     }
// }
//
