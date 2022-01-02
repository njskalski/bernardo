// use crate::{AnyMsg, InputEvent, Output, Theme, Widget};
// use crate::experiments::scroll::Scroll;
// use crate::primitives::xy::XY;
// use crate::widget::widget::WID;
//
// struct WithScroll<W: Widget> {
//     widget: W,
//     scroll: Scroll,
// }
//
// impl<W: Widget> WithScroll<W> {
//     pub fn new(widget: W) -> Self {
//         Self {
//             widget,
//             scroll: Scroll::new(XY::new(100, 1000)),
//         }
//     }
// }
//
// impl<W: Widget> Widget for WithScroll<W> {
//     fn id(&self) -> WID {
//         self.widget.id()
//     }
//
//     fn typename(&self) -> &'static str {
//         self.widget.typename()
//     }
//
//     fn min_size(&self) -> XY {
//         self.widget.min_size()
//     }
//
//     fn layout(&mut self, max_size: XY) -> XY {
//         todo!()
//     }
//
//     fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
//         self.widget.on_input(input_event)
//     }
//
//     fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
//         self.widget.update(msg)
//     }
//
//     fn get_focused(&self) -> Option<&dyn Widget> {
//         self.widget.get_focused()
//     }
//
//     fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
//         self.widget.get_focused_mut()
//     }
//
//     fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
//         todo!()
//     }
//
//     fn anchor(&self) -> XY {
//         todo!()
//     }
//
//     fn subwidgets_mut(&mut self) -> Box<dyn Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
//         todo!()
//     }
//
//     fn subwidgets(&self) -> Box<dyn Iterator<Item=&dyn Widget> + '_> where Self: Sized {
//         todo!()
//     }
//
//     fn get_subwidget(&self, wid: WID) -> Option<&dyn Widget> where Self: Sized {
//         todo!()
//     }
//
//     fn get_subwidget_mut(&mut self, wid: WID) -> Option<&mut dyn Widget> where Self: Sized {
//         todo!()
//     }
// }