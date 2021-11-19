use crate::io::output::Output;
use crate::io::style::TextStyle;
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

enum WallState {
    // Don't draw that wall at all. Draw connecting pieces instead of corners.
    Skip,
    // Draw this wall as independent one, including corners.
    Lone,
    // Draw this wall as one that will be connected from beyond the given rectangle.
    // Instead of freestanding corners, draw double-corners.
    Connecting,
}

struct WallStates {
    pub Left: WallState,
    pub Top: WallState,
    pub Right: WallState,
    pub Bottom: WallState,
}

impl WallStates {
    pub fn get_corner_top_left(wall_states: &WallStates, border_style: &BorderStyle) -> Option<&'static str> {
        match (&wall_states.Left, &wall_states.Top) {
            (WallState::Skip, WallState::Skip) => None,
            (WallState::Lone, WallState::Skip) => Some(border_style.VerticalLine),
            (WallState::Skip, WallState::Lone) => Some(border_style.HorizontalLine),
            (WallState::Lone, WallState::Lone) => Some(border_style.UpperLeft),
            (WallState::Lone, WallState::Connecting) => Some(border_style.CrossNoLeft),
            (WallState::Connecting, WallState::Lone) =>
        }
    }
}

pub fn draw_some(rect: Rect, wall_states: WallStates, text_style: TextStyle, border_style: &BorderStyle, output: &mut Output) {
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
