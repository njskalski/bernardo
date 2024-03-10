/*
All these structures are design with test in mind *only*. Meaning they are allowed to panic and
be slow to a reasonable degree.
 */

#![allow(dead_code)]

pub mod button_interpreter;
pub mod code_results_interpreter;
pub mod completion_interpreter;
pub mod context_bar_interpreter;
pub mod editbox_interpreter;
pub mod editor_interpreter;
pub mod full_setup;
pub mod fuzzy_search_interpreter;
pub mod listview_interpreter;
mod log_capture;
pub mod meta_frame;
pub mod mock_clipboard;
pub mod mock_input;
pub mod mock_labels_provider;
pub mod mock_navcomp_loader;
pub mod mock_navcomp_promise;
pub mod mock_navcomp_provider;
pub mod mock_output;
pub mod no_editor_interpreter;
pub mod savefile_interpreter;
pub mod scroll_interpreter;
pub mod treeview_interpreter;
pub mod with_scroll_interpreter;
pub mod nested_menu_interpreter;
