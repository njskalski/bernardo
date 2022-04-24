use crate::{Output, ZERO};
use crate::io::style::TextStyle;
use crate::primitives::xy::XY;

pub struct BorderStyle {
    pub upper_left: &'static str,
    pub upper_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal_line: &'static str,
    pub vertical_line: &'static str,
    pub full_cross: &'static str,
    pub cross_no_left: &'static str,
    pub cross_no_top: &'static str,
    pub cross_no_right: &'static str,
    pub cross_no_bottom: &'static str,
}

pub const SINGLE_BORDER_STYLE: BorderStyle = BorderStyle {
    upper_left: "┌",
    upper_right: "┐",
    bottom_left: "└",
    bottom_right: "┘",
    horizontal_line: "─",
    vertical_line: "│",
    full_cross: "┼",
    cross_no_left: "├",
    cross_no_top: "┬",
    cross_no_right: "┤",
    cross_no_bottom: "┴",
};

impl BorderStyle {
    pub fn draw_edges(&self, style: TextStyle, output: &mut dyn Output) {
        draw_full_rect(style, self, output)
    }
}


fn draw_full_rect(style: TextStyle, border_style: &BorderStyle, output: &mut dyn Output) {
    let size = output.size_constraint().visible_hint().size;
    if size > XY::new(1, 1) {
        output.print_at(ZERO,
                        style,
                        border_style.upper_left);
        output.print_at(XY::new(0, size.y - 1),
                        style,
                        border_style.bottom_left);
        output.print_at(XY::new(size.x - 1, 0),
                        style,
                        border_style.upper_right);
        output.print_at(XY::new(size.x - 1, size.y - 1),
                        style,
                        border_style.bottom_right);

        for x in 1..size.x - 1 {
            output.print_at(XY::new(x, 0),
                            style,
                            border_style.horizontal_line);
            output.print_at(XY::new(x, size.y - 1),
                            style,
                            border_style.horizontal_line);
        }

        for y in 1..size.y - 1 {
            output.print_at(XY::new(0, y),
                            style,
                            border_style.vertical_line);
            output.print_at(XY::new(size.x - 1, y),
                            style,
                            border_style.vertical_line);
        }
    } else {
        for x in 0..size.x {
            for y in 0..size.y {
                output.print_at(
                    XY::new(x, y),
                    style,
                    "╳",
                );
            }
        }
    }
}
//
// pub fn draw_some(wirs: &Vec<WidgetIdRect>, text_style: TextStyle, border_style: &BorderStyle, output: &mut Output) {
//     if output.size() > XY::new(2, 2) {
//         let mut corner_neighbours = HashSet::<XY>::new();
//         let mut corners = HashSet::<XY>::new();
//
//         for wir in wirs {
//             let rect = &wir.rect;
//
//             for x in rect.pos.x..rect.lower_right().x {
//                 output.print_at(XY::new(x, rect.pos.y),
//                                 text_style,
//                                 border_style.vertical_line);
//                 output.print_at(XY::new(x, rect.lower_right().y - 1),
//                                 text_style,
//                                 border_style.vertical_line);
//             }
//
//             for y in 0..output.size().y {
//                 output.print_at(XY::new(rect.pos.x, y),
//                                 text_style,
//                                 border_style.horizontal_line);
//                 output.print_at(XY::new(rect.lower_right().x - 1, y),
//                                 text_style,
//                                 border_style.horizontal_line);
//             }
//
//             for c in rect.corners() {
//                 if !(c < output.size()) {
//                     warn!("corner {} beyond border of output {} ", c, output.size());
//                     continue;
//                 }
//                 corners.insert(c);
//                 for n in c.neighbours() {
//                     if n < output.size() {
//                         corner_neighbours.insert(n);
//                     }
//                 }
//             }
//         }
//
//         for corner in corners {
//             let present: Vec<bool> = corner.neighbours().map(
//                 |n| corner_neighbours.contains(n)
//             ).collect();
//         }
//     } else {
//         for wir in wirs {
//             let rect = &wir.rect;
//
//             for x in rect.pos.x..rect.lower_right().x {
//                 for y in rect.pos.y..rect.lower_right().y {
//                     let pos = XY::new(x, y);
//                     if pos < output.size() {
//                         output.print_at(
//                             pos,
//                             text_style,
//                             "╳",
//                         );
//                     }
//                 }
//             }
//         }
//     }
// }
