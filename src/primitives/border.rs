use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use log::error;
use std::cmp::min;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    pub fn draw_output_edges(&self, style: TextStyle, output: &mut dyn Output, label_op: Option<&str>) {
        if output.size() > XY::new(2, 2) {
            let output_size = output.size();
            let rect = Rect::new(XY::new(0, 0), XY::new(output_size.x - 1, output_size.y - 1));
            self.draw_full_rect(style, output, rect, label_op);
        } else {
            error!("skipping draw_output_edges - output size {} is too small", output.size());
        }
    }

    pub fn draw_full_rect(&self, style: TextStyle, output: &mut dyn Output, rect: Rect, label: Option<&str>) {
        let output_size = output.size();
        let border_style = self;

        if rect.lower_right() > output_size {
            error!("skipping draw_full_rect - rect {:?} does not fit output size {}", rect, output_size);
            return;
        }

        if rect.size > XY::new(1, 1) {
            let label = label.unwrap_or("");
            let label_width = min(label.width(), 20) as u16;
            let label_pos_x = 3 as u16;

            output.print_at(rect.pos, style, border_style.upper_left);
            output.print_at(rect.pos + XY::new(0, rect.size.y), style, border_style.bottom_left);
            output.print_at(rect.pos + XY::new(rect.size.x, 0), style, border_style.upper_right);
            output.print_at(rect.pos + XY::new(rect.size.x, rect.size.y), style, border_style.bottom_right);

            if !label.is_empty() {
                output.print_at(rect.pos + XY::new(label_pos_x, 0), style, label);
            }

            for x in 1..rect.size.x {
                let under_label = x >= label_pos_x && x < (label_pos_x + label_width);

                if !under_label {
                    output.print_at(rect.pos + XY::new(x, 0), style, border_style.horizontal_line);
                }

                output.print_at(rect.pos + XY::new(x, rect.size.y), style, border_style.horizontal_line);
            }

            for y in 1..rect.size.y {
                output.print_at(rect.pos + XY::new(0, y), style, border_style.vertical_line);
                output.print_at(rect.pos + XY::new(rect.size.x, y), style, border_style.vertical_line);
            }
        } else {
            for x in rect.upper_left().x..rect.lower_right().x {
                for y in rect.upper_left().y..rect.lower_right().y {
                    output.print_at(XY::new(x, y), style, "╳");
                }
            }
        }
    }
}

// Here the Rect is INCLUSIVE

//
// pub fn draw_some(wirs: &Vec<WidgetIdRect>, text_style: TextStyle, border_style: &BorderStyle,
// output: &mut Output) {     if output.size() > XY::new(2, 2) {
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
