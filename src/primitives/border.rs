use std::collections::HashSet;

use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::layout::layout::WidgetIdRect;
use crate::primitives::rect::Rect;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};

pub struct BorderStyle {
    pub UpperLeft: &'static str,
    pub UpperRight: &'static str,
    pub BottomLeft: &'static str,
    pub BottomRight: &'static str,
    pub HorizontalLine: &'static str,
    pub VerticalLine: &'static str,
    pub FullCross: &'static str,
    pub CrossNoLeft: &'static str,
    pub CrossNoTop: &'static str,
    pub CrossNoRight: &'static str,
    pub CrossNoBottom: &'static str,
}

pub const SingleBorderStyle: BorderStyle = BorderStyle {
    UpperLeft: "┌",
    UpperRight: "┐",
    BottomLeft: "└",
    BottomRight: "┘",
    HorizontalLine: "─",
    VerticalLine: "│",
    FullCross: "┼",
    CrossNoLeft: "├",
    CrossNoTop: "┬",
    CrossNoRight: "┤",
    CrossNoBottom: "┴",
};


pub fn draw_full_rect(style: TextStyle, border_style: &BorderStyle, output: &mut Output) {
    if output.size() > XY::new(1, 1) {
        output.print_at(ZERO,
                        style,
                        border_style.UpperLeft);
        output.print_at(XY::new(0, output.size().y - 1),
                        style,
                        border_style.BottomLeft);
        output.print_at(XY::new(output.size().x - 1, 0),
                        style,
                        border_style.UpperRight);
        output.print_at(XY::new(output.size().x - 1, output.size().y - 1),
                        style,
                        border_style.BottomRight);

        for x in 0..output.size().x {
            output.print_at(XY::new(x, 0),
                            style,
                            border_style.VerticalLine);
            output.print_at(XY::new(x, output.size().y - 1),
                            style,
                            border_style.VerticalLine);
        }

        for y in 0..output.size().y {
            output.print_at(XY::new(0, y),
                            style,
                            border_style.HorizontalLine);
            output.print_at(XY::new(output.size().x - 1, y),
                            style,
                            border_style.HorizontalLine);
        }
    } else {
        for x in 0..output.size().x {
            for y in 0..output.size().y {
                output.print_at(
                    XY::new(x, y),
                    style,
                    "╳",
                );
            }
        }
    }
}

pub fn draw_some(wirs: &Vec<WidgetIdRect>, text_style: TextStyle, border_style: &BorderStyle, output: &mut Output) {
    if output.size() > XY::new(2, 2) {
        let mut corner_neighbours = HashSet::<XY>::new();
        let mut corners = HashSet::<XY>::new();

        for wir in wirs {
            let rect = &wir.rect;

            for x in rect.pos.x..rect.lower_right().x {
                output.print_at(XY::new(x, rect.pos.y),
                                text_style,
                                border_style.VerticalLine);
                output.print_at(XY::new(x, rect.lower_right().y - 1),
                                text_style,
                                border_style.VerticalLine);
            }

            for y in 0..output.size().y {
                output.print_at(XY::new(rect.pos.x, y),
                                text_style,
                                border_style.HorizontalLine);
                output.print_at(XY::new(rect.lower_right().x - 1, y),
                                text_style,
                                border_style.HorizontalLine);
            }

            for c in rect.corners() {
                corners.insert(c);
                for n in c.neighbours() {
                    if n < output.size() {
                        corner_neighbours.insert(n);
                    }
                }
            }
        }
    } else {
        for x in rect.pos.x..rect.lower_right().x {
            for y in rect.pos.y..rect.lower_right().y {
                output.print_at(
                    XY::new(x, y),
                    text_style,
                    "╳",
                );
            }
        }
    }
}
