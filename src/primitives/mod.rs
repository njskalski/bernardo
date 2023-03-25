pub mod arrow;
pub mod color;
pub mod cursor_set;
mod cursor_set_tests;
pub mod rect;
pub mod sized_xy;
pub mod styled_string;
pub mod xy;
pub mod helpers;
pub mod border;
pub mod size_constraint;

mod rope_buffer_state;
mod cursor_set_selection_tests;
pub mod scroll;
pub mod cursor_set_rect;
pub mod alphabet;

pub mod is_default;
pub mod tmtheme;
pub mod common_edit_msgs;
mod common_edit_msgs_tests;
pub mod search_pattern;
pub mod provider;
pub mod macros;
pub mod common_query;
pub mod stupid_cursor;

pub mod cursor_set_fuzz;
pub mod scroll_enum;
pub mod has_invariant;
pub mod printable;
pub mod cursor;