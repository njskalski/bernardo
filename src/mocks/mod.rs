/*
All these structures are design with test in mind *only*. Meaning they are allowed to panic and
be slow to a reasonable degree.
 */

pub mod mock_output;
pub mod mock_input;
pub mod mock_navcomp_provider;
pub mod mock_clipboard;
pub mod full_setup;
pub mod mock_navcomp_promise;
pub mod editor_interpreter;
pub mod scroll_interpreter;
pub mod completion_interpreter;
pub mod meta_frame;
pub mod savefile_interpreter;
pub mod treeview_interpreter;
pub mod listview_interpreter;
pub mod editbox_interpreter;
pub mod button_interpreter;
pub mod no_editor_interpreter;
pub mod fuzzy_search_interpreter;
