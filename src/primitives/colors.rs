// Some hardcoded colors, I will implement user settings later.

use crate::primitives::color::Color;

// pub const COLOR_TUNA: Color = Color::new(235, 111, 146);
// pub const COLOR_BRIGHT_BACKGROUND: Color = Color::new(25, 23, 36);
// pub const COLOR_DARK_BACKGROUND: Color = Color::new(12, 11, 18);
//
// pub const COLOR_INACTIVE_INPUT_FOREGROUND: Color = Color::new(44, 34, 55);
// pub const COLOR_INACTIVE_INPUT_BACKGROUND: Color = Color::new(14, 14, 27);
//
// pub const COLOR_ACTIVE_INPUT_FOREGROUND: Color = Color::new(70, 81, 90);
// pub const COLOR_ACTIVE_INPUT_BACKGROUND: Color = Color::new(35, 39, 68);
//
// pub const COLOR_NONINTERACTIVE_TEXT_FOCUSED: Color = Color::new(33, 53, 79);
// pub const COLOR_NONINTERACTIVE_TEXT_NOT_FOCUSED: Color = Color::new(16, 26, 39);
//
// pub const COLOR_VIOLET: Color = Color::new(110, 106, 134);
// pub const COLOR_BRIGHT_VIOLET: Color = Color::new(196, 167, 231);
// pub const COLOR_WATER: Color = Color::new(49, 116, 143);
// pub const COLOR_SCRAMBLED_EGGS: Color = Color::new(246, 193, 119);
// pub const COLOR_SKY: Color = Color::new(156, 207, 216);
// pub const COLOR_PINK_SAND: Color = Color::new(235, 188, 186);

pub const COLOR_CURSOR_BACKGROUND: Color = Color::new(90, 47, 61);
pub const COLOR_CURSOR_FOREGROUND: Color = Color::new(10, 9, 16);

pub const DEFAULT_TEXT_BACKGROUND: Color = Color::new(10, 9, 16);
pub const DEFAULT_TEXT_FOREGROUND: Color = Color::new(79, 69, 92);

pub const EDIT_BACKGROUND: Color = Color::new(15, 13, 20);
pub const EDIT_FOREGROUND: Color = Color::new(79, 69, 93);

pub const HEADER_FOREGROUND: Color = Color::new(26, 50, 61);
pub const HEADER_BACKGROUND: Color = Color::new(10, 9, 16);