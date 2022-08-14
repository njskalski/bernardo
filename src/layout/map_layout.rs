// use crate::experiments::subwidget_pointer::{SubwidgetPointer, SubwidgetPointerOp};
// use crate::layout::layout::{Layout, WidgetIdRect};
// use crate::primitives::xy::XY;
// use crate::Widget;
//
// struct MapLayout<W: Widget> {
//     getter_op: SubwidgetPointerOp<W>,
//     transformator: Box<dyn Fn(Box<dyn Layout<W>>, Box<dyn Layout<W>>) -> Box<dyn Layout<W>>>,
//     root_layout: Box<dyn Layout<W>>,
// }
//
// impl<W: Widget> Layout<W> for MapLayout<W> {
//     fn min_size(&self, root: &W) -> XY {
//         match self.getter_op.get(root).map(|f| ) {
//             None => {
//                 self.root_layout.min_size(root)
//             }
//             Some(widget) => {
//                 Le
//
//             }
//         }
//     }
//
//     fn calc_sizes(&self, root: &mut W, output_size: XY) -> Vec<WidgetIdRect> {
//         todo!()
//     }
// }