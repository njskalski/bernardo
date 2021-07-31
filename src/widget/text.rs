// use crate::widget::widget::Widget;
// use crate::primitives::color::{Color, BLACK, WHITE};
// use crate::io::input_event::InputEvent;
//
// struct Text {
//     background : Color,
//     foreground: Color,
//     text : String,
//     blank_prefix : u16,
//     blank_suffix : u16,
// }
//
// impl Text {
//     fn simple(text : String) -> Self {
//         Text{
//             background: BLACK,
//             foreground: WHITE,
//             text,
//             blank_prefix: 0,
//             blank_suffix: 0,
//         }
//     }
// }
//
// impl Widget for Text {
//     fn focusable(&self) -> bool {
//         false
//     }
//
//     fn on_input(&self, _: InputEvent) -> bool {
//         false
//     }
// }