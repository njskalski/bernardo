use log::warn;
use regex::Regex;

use crate::widgets::editor_widget::label::label::{Label, LabelPos, LabelStyle};

/*
TODO this is a very incomplete implementation.
- it does not handle errors
- it does not use settings at all, so no way to fine tune the display behavior
*/

/*
Generated with chat-gpt:

        let warning_message = capture.get(1).map_or("", |m| m.as_str());
        let filename = capture.get(2).map_or("", |m| m.as_str());
        let line_number = capture.get(3).map_or("", |m| m.as_str());
        let column_number = capture.get(4).map_or("", |m| m.as_str());
 */
const WARNING_PATTERN_STR: &str = r"warning:.+[\r\n]+\s+-->\s+([^:]+?):([0-9]+):([0-9]+)[\r\n]+";
const ERROR_PATTERN_STR: &str = r"error[^:]*:[^:]*[\r\n]+\s+-->\s+([^:]+?):([0-9]+):([0-9]+)[\r\n]+";

// TODO I actually think this should be written in Python in a separate program, first of "plugins",
// to test out the idea.
pub struct RustcOutputParserLabelProvider {
    warning_regex: Regex,
    error_regex: Regex,

    labels: Vec<Label>,
}

impl RustcOutputParserLabelProvider {
    pub fn new() -> Self {
        let warning_regex = Regex::new(WARNING_PATTERN_STR).expect("Failed to compile regex pattern"); //TODO?
        let error_regex = Regex::new(ERROR_PATTERN_STR).expect("Failed to compile regex pattern"); //TODO?

        RustcOutputParserLabelProvider {
            warning_regex,
            error_regex,
            labels: vec![],
        }
    }

    pub fn errors_iter(&self) -> impl Iterator<Item=&Label> {
        self.labels.iter().filter(|label| label.style == LabelStyle::Error)
    }

    pub fn warnings_iter(&self) -> impl Iterator<Item=&Label> {
        self.labels.iter().filter(|label| label.style == LabelStyle::Warning)
    }

    pub fn ingest(&mut self, rustc_output: &str) -> bool {
        let mut new_labels: Vec<Label> = vec![];

        for capture in self.warning_regex.captures_iter(rustc_output) {
            let warning_message = match capture.get(0).map(|m| m.as_str()) {
                Some(msg) => msg,
                None => {
                    warn!("skipping: no warning message");
                    continue;
                }
            };

            let _filename = match capture.get(1).map(|m| m.as_str()) {
                Some(msg) => msg,
                None => {
                    warn!("skipping warning: no filename");
                    continue;
                }
            };

            let line_number: usize = match capture.get(2).map(|m| m.as_str()).map(|s| s.parse::<usize>().ok()).flatten() {
                Some(idx) => idx,
                None => {
                    warn!("skipping warning: no line number");
                    continue;
                }
            };

            let _col_number: usize = match capture.get(3).map(|m| m.as_str()).map(|s| s.parse::<usize>().ok()).flatten() {
                Some(idx) => idx,
                None => {
                    warn!("skipping warning: no column number");
                    continue;
                }
            };

            new_labels.push(Label::new(
                LabelPos::LineAfter { line_no_1b: line_number },
                LabelStyle::Warning,
                Box::new(warning_message.to_string()),
            ));
        }

        for capture in self.error_regex.captures_iter(rustc_output) {
            let error_message = match capture.get(0).map(|m| m.as_str()) {
                Some(msg) => msg,
                None => {
                    warn!("skipping: no error message");
                    continue;
                }
            };

            let _filename = match capture.get(1).map(|m| m.as_str()) {
                Some(msg) => msg,
                None => {
                    warn!("skipping error: no filename");
                    continue;
                }
            };

            let line_number: usize = match capture.get(2).map(|m| m.as_str()).map(|s| s.parse::<usize>().ok()).flatten() {
                Some(idx) => idx,
                None => {
                    warn!("skipping error: no line number");
                    continue;
                }
            };

            let _col_number: usize = match capture.get(3).map(|m| m.as_str()).map(|s| s.parse::<usize>().ok()).flatten() {
                Some(idx) => idx,
                None => {
                    warn!("skipping error: no column number");
                    continue;
                }
            };

            new_labels.push(Label::new(
                LabelPos::LineAfter { line_no_1b: line_number },
                LabelStyle::Error,
                Box::new(error_message.to_string()),
            ));
        }

        self.labels = new_labels;

        true
    }
}

// impl LabelsProvider for RustcOutputParserLabelProvider {
//     fn query_for(&self, path_op: Option<SPath>, buffer: &dyn TextBuffer, char_range:
// Range<usize>) -> Box<dyn Iterator<Item=&'_ Label> + '_> {         todo!()
//     }
// }

#[cfg(test)]
pub mod test {
    use crate::widgets::editor_widget::label::rustc_output_parser_label_provider::RustcOutputParserLabelProvider;

    #[test]
    fn rustc_warnings_test() {
        let mut rustc_output_parser = RustcOutputParserLabelProvider::new();
        assert!(rustc_output_parser.ingest(SOME_OUTPUT_1));
        assert_eq!(rustc_output_parser.warnings_iter().count(), 132);
        assert_eq!(rustc_output_parser.errors_iter().count(), 0);
    }

    #[test]
    fn rustc_warnings_and_error_test() {
        let mut rustc_output_parser = RustcOutputParserLabelProvider::new();
        assert!(rustc_output_parser.ingest(SOME_OUTPUT_2));
        assert_eq!(rustc_output_parser.warnings_iter().count(), 79);
        assert_eq!(rustc_output_parser.errors_iter().count(), 1);
    }

    const SOME_OUTPUT_1: &str = r###"
warning: unused import: `Metadata`
  --> src/io/buffer_output/buffer_output.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::string::String`
 --> src/io/buffer_output/buffer_output_cells_iter.rs:1:5
  |
1 | use std::string::String;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::io::style::TextStyle`
 --> src/io/buffer_output/buffer_output_cells_iter.rs:5:5
  |
5 | use crate::io::style::TextStyle;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
 --> src/io/buffer_output/buffer_output_cells_iter.rs:6:5
  |
6 | use crate::primitives::rect::Rect;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/io/over_output.rs:8:25
  |
8 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::unpack_or`
  --> src/io/over_output.rs:12:5
   |
12 | use crate::unpack_or;
   |     ^^^^^^^^^^^^^^^^

warning: unused import: `debug`
 --> src/io/sub_output.rs:3:11
  |
3 | use log::{debug, error};
  |           ^^^^^

warning: unused import: `Metadata`
 --> src/io/sub_output.rs:6:25
  |
6 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `Metadata`
  --> src/io/crossterm_output.rs:12:38
   |
12 | use crate::io::output::{FinalOutput, Metadata, Output};
   |                                      ^^^^^^^^

warning: unused import: `crate::cursor::cursor_set::CursorSet`
  --> src/text/buffer_state_fuzz.rs:10:5
   |
10 | use crate::cursor::cursor_set::CursorSet;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `warn`
 --> src/text/contents_and_cursors.rs:3:25
  |
3 | use log::{debug, error, warn};
  |                         ^^^^

warning: unused import: `crate::io::buffer`
  --> src/text/contents_and_cursors.rs:11:5
   |
11 | use crate::io::buffer;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `std::cell::RefCell`
 --> src/tsw/tree_sitter_wrapper.rs:1:5
  |
1 | use std::cell::RefCell;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::rc::Rc`
 --> src/tsw/tree_sitter_wrapper.rs:5:5
  |
5 | use std::rc::Rc;
  |     ^^^^^^^^^^^

warning: unused import: `std::cell::RefCell`
 --> src/tsw/parsing_tuple.rs:1:5
  |
1 | use std::cell::RefCell;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::rc::Rc`
 --> src/tsw/parsing_tuple.rs:3:5
  |
3 | use std::rc::Rc;
  |     ^^^^^^^^^^^

warning: unused import: `std::cell::Cell`
 --> src/w7e/navcomp_provider_lsp.rs:1:5
  |
1 | use std::cell::Cell;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `DocumentSymbolResponse`
 --> src/w7e/navcomp_provider_lsp.rs:8:57
  |
8 | use lsp_types::{CompletionResponse, CompletionTextEdit, DocumentSymbolResponse, Position, SymbolKind};
  |                                                         ^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `NavCompSymbol`, `SymbolPromise`
  --> src/w7e/navcomp_provider_lsp.rs:19:122
   |
19 | use crate::w7e::navcomp_provider::{Completion, CompletionAction, CompletionsPromise, FormattingPromise, NavCompProvider, NavCompSymbol, StupidSubstituteMessage, SymbolContextActionsPromise, SymbolPromise, SymbolType, SymbolUsage, SymbolUsagesPromise};
   |                                                                                                                          ^^^^^^^^^^^^^                                                        ^^^^^^^^^^^^^

warning: unused imports: `Arc`, `RwLock`, `TryLockResult`
 --> src/w7e/workspace.rs:2:17
  |
2 | use std::sync::{Arc, RwLock, TryLockResult};
  |                 ^^^  ^^^^^^  ^^^^^^^^^^^^^

warning: unused imports: `NavCompGroupRef`, `NavCompGroup`
  --> src/w7e/workspace.rs:13:33
   |
13 | use crate::w7e::navcomp_group::{NavCompGroup, NavCompGroupRef};
   |                                 ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/save_file_dialog/save_file_dialog.rs:22:25
   |
22 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/tree_view/tree_view.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/tree_view/tree_view.rs:16:5
   |
16 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/button.rs:8:25
  |
8 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/button.rs:10:5
   |
10 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/edit_box.rs:11:25
   |
11 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/edit_box.rs:14:5
   |
14 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/no_editor.rs:5:25
  |
5 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
 --> src/widgets/no_editor.rs:6:5
  |
6 | use crate::primitives::rect::Rect;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::unpack_or`
 --> src/widgets/no_editor.rs:9:5
  |
9 | use crate::unpack_or;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/with_scroll.rs:5:25
  |
5 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> src/widgets/main_view/main_view.rs:1:5
  |
1 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::io::loading_state::LoadingState`
  --> src/widgets/main_view/main_view.rs:16:5
   |
16 | use crate::io::loading_state::LoadingState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::promise::promise::PromiseState`
  --> src/widgets/main_view/main_view.rs:26:5
   |
26 | use crate::promise::promise::PromiseState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::text::buffer_state::BufferState`
  --> src/widgets/main_view/main_view.rs:27:5
   |
27 | use crate::text::buffer_state::BufferState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::buffer_state_shared_ref::BufferSharedRef`
  --> src/widgets/main_view/main_view.rs:28:5
   |
28 | use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::navcomp_group::NavCompGroupRef`
  --> src/widgets/main_view/main_view.rs:29:5
   |
29 | use crate::w7e::navcomp_group::NavCompGroupRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `either::Either`
 --> src/widgets/main_view/msg.rs:1:5
  |
1 | use either::Either;
  |     ^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::navcomp_provider::SymbolUsagesPromise`
 --> src/widgets/main_view/msg.rs:6:5
  |
6 | use crate::w7e::navcomp_provider::SymbolUsagesPromise;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::RwLockWriteGuard`
 --> src/widgets/editor_widget/editor_widget.rs:2:5
  |
2 | use std::sync::RwLockWriteGuard;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/editor_widget/editor_widget.rs:20:25
   |
20 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::tsw::tree_sitter_wrapper::HighlightItem`
  --> src/widgets/editor_widget/editor_widget.rs:35:5
   |
35 | use crate::tsw::tree_sitter_wrapper::HighlightItem;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `NavCompSymbol`
  --> src/widgets/editor_widget/editor_widget.rs:38:54
   |
38 | use crate::w7e::navcomp_provider::{CompletionAction, NavCompSymbol};
   |                                                      ^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/editor_widget/completion/completion_widget.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/editor_widget/completion/completion_widget.rs:17:5
   |
17 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/editor_widget/context_bar/widget.rs:7:25
  |
7 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/editor_widget/context_bar/widget.rs:12:5
   |
12 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::ops::Range`
 --> src/widgets/editor_widget/label/label.rs:1:5
  |
1 | use std::ops::Range;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `crate::text::text_buffer::TextBuffer`
 --> src/widgets/editor_widget/label/label.rs:6:5
  |
6 | use crate::text::text_buffer::TextBuffer;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::widgets::editor_widget::label::labels_provider::LabelsProvider`
 --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:6:5
  |
6 | use crate::widgets::editor_widget::label::labels_provider::LabelsProvider;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `unpack_or`
 --> src/widgets/fuzzy_search/fuzzy_search.rs:7:13
  |
7 | use crate::{unpack_or, unpack_or_e};
  |             ^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/fuzzy_search/fuzzy_search.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/editor_view/editor_view.rs:10:25
   |
10 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::w7e::handler::NavCompRef`
  --> src/widgets/editor_view/editor_view.rs:24:5
   |
24 | use crate::w7e::handler::NavCompRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::navcomp_group::NavCompGroupRef`
  --> src/widgets/editor_view/editor_view.rs:25:5
   |
25 | use crate::w7e::navcomp_group::NavCompGroupRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::widgets::main_view::main_view::DocumentIdentifier`
  --> src/widgets/editor_view/editor_view.rs:32:5
   |
32 | use crate::widgets::main_view::main_view::DocumentIdentifier;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/list_widget/list_widget.rs:12:25
   |
12 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/list_widget/list_widget.rs:17:5
   |
17 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::rc::Rc`
 --> src/widgets/code_results_view/code_results_widget.rs:3:5
  |
3 | use std::rc::Rc;
  |     ^^^^^^^^^^^

warning: unused import: `std::str::from_utf8`
 --> src/widgets/code_results_view/code_results_widget.rs:4:5
  |
4 | use std::str::from_utf8;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::RwLockWriteGuard`
 --> src/widgets/code_results_view/code_results_widget.rs:5:5
  |
5 | use std::sync::RwLockWriteGuard;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `either::Left`
 --> src/widgets/code_results_view/code_results_widget.rs:7:5
  |
7 | use either::Left;
  |     ^^^^^^^^^^^^

warning: unused import: `regex::internal::Input`
 --> src/widgets/code_results_view/code_results_widget.rs:9:5
  |
9 | use regex::internal::Input;
  |     ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::cursor::cursor::Cursor`
  --> src/widgets/code_results_view/code_results_widget.rs:13:5
   |
13 | use crate::cursor::cursor::Cursor;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::fs::read_error::ReadError`
  --> src/widgets/code_results_view/code_results_widget.rs:16:5
   |
16 | use crate::fs::read_error::ReadError;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/code_results_view/code_results_widget.rs:21:25
   |
21 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/code_results_view/code_results_widget.rs:25:5
   |
25 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::text::buffer_state::BufferState`
  --> src/widgets/code_results_view/code_results_widget.rs:29:5
   |
29 | use crate::text::buffer_state::BufferState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::buffer_state_shared_ref::BufferSharedRef`
  --> src/widgets/code_results_view/code_results_widget.rs:30:5
   |
30 | use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `UpdateResult`
 --> src/widgets/code_results_view/symbol_usage_promise_provider:8:45
  |
8 | use crate::promise::promise::{PromiseState, UpdateResult};
  |                                             ^^^^^^^^^^^^

warning: unused import: `crate::fs::path::SPath`
 --> src/widgets/code_results_view/code_results_msg.rs:3:5
  |
3 | use crate::fs::path::SPath;
  |     ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::widgets::main_view::main_view::DocumentIdentifier`
 --> src/widgets/code_results_view/code_results_msg.rs:5:5
  |
5 | use crate::widgets::main_view::main_view::DocumentIdentifier;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/big_list/big_list_widget.rs:6:25
  |
6 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `std::collections::HashMap`
  --> src/cursor/cursor.rs:23:5
   |
23 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `IterMut`, `Iter`
  --> src/cursor/cursor.rs:25:18
   |
25 | use std::slice::{Iter, IterMut};
   |                  ^^^^  ^^^^^^^

warning: unused import: `unicode_segmentation::UnicodeSegmentation`
 --> src/widgets/editor_widget/editor_widget.rs:7:5
  |
7 | use unicode_segmentation::UnicodeSegmentation;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::primitives::printable::Printable`
 --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:4:5
  |
4 | use crate::primitives::printable::Printable;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Promise`
  --> src/widgets/editor_widget/editor_widget.rs:32:31
   |
32 | use crate::promise::promise::{Promise, PromiseState};
   |                               ^^^^^^^

warning: unused variable: `size`
   --> src/widgets/save_file_dialog/save_file_dialog.rs:434:13
    |
434 |         let size = unpack_or_e!(self.display_state.as_ref(), (), "render before layout").total_size;
    |             ^^^^ help: if this is intentional, prefix it with an underscore: `_size`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `size`
   --> src/widgets/tree_view/tree_view.rs:333:13
    |
333 |         let size = unpack_or_e!(self.last_size, (), "render before layout");
    |             ^^^^ help: if this is intentional, prefix it with an underscore: `_size`

warning: unused variable: `size`
  --> src/widgets/button.rs:90:13
   |
90 |         let size = XY::new(unpack_or!(self.last_size_x, (), "render before layout"), 1);
   |             ^^^^ help: if this is intentional, prefix it with an underscore: `_size`

warning: unused variable: `input_event`
   --> src/widgets/with_scroll.rs:236:24
    |
236 |     fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
    |                        ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_input_event`

warning: unused variable: `opened`
   --> src/widgets/main_view/main_view.rs:118:32
    |
118 |             buffer_shared_ref, opened
    |                                ^^^^^^ help: try ignoring the field: `opened: _`

warning: unused variable: `navcomp`
   --> src/widgets/editor_widget/editor_widget.rs:268:19
    |
268 |             (Some(navcomp), None) => {
    |                   ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_navcomp`

warning: unused variable: `path`
   --> src/widgets/editor_widget/editor_widget.rs:888:13
    |
888 |         let path = unpack_or!(buffer.get_path(), (), "no path set");
    |             ^^^^ help: if this is intentional, prefix it with an underscore: `_path`

warning: unused variable: `stupid_cursor`
   --> src/widgets/editor_widget/editor_widget.rs:889:13
    |
889 |         let stupid_cursor = unpack_or!(StupidCursor::from_real_cursor(buffer, cursor).ok(), (), "failed conversion to stupid cursor");
    |             ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_stupid_cursor`

warning: unused variable: `filename`
  --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:64:17
   |
64 |             let filename = match capture.get(2).map(|m| m.as_str()) {
   |                 ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_filename`

warning: unused variable: `col_number`
  --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:80:17
   |
80 |             let col_number: usize = match capture.get(4).map(|m| m.as_str()).map(|s| s.parse::<usize>().ok()).flatten() {
   |                 ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_col_number`

warning: unused variable: `debug_text`
  --> src/widgets/text_widget.rs:29:13
   |
29 |         let debug_text = self.text.to_string();
   |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_debug_text`

warning: unused variable: `total_size`
   --> src/widgets/editor_view/editor_view.rs:443:13
    |
443 |         let total_size = unpack_or_e!(
    |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_total_size`

warning: unused variable: `size`
   --> src/widgets/list_widget/list_widget.rs:356:13
    |
356 |         let size = unpack_or_e!(self.last_size, (), "render before layout");
    |             ^^^^ help: if this is intentional, prefix it with an underscore: `_size`

warning: unused variable: `new_line_is_last`
   --> src/cursor/cursor_set.rs:304:17
    |
304 |             let new_line_is_last = new_line_idx + 1 == rope.len_lines();
    |                 ^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_new_line_is_last`

warning: variable does not need to be mutable
   --> src/experiments/buffer_register.rs:117:13
    |
117 |         let mut seen_refs: HashSet<SPath> = HashSet::new();
    |             ----^^^^^^^^^
    |             |
    |             help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` on by default

warning: variable does not need to be mutable
   --> src/widgets/main_view/main_view.rs:120:13
    |
120 |         let mut buffer_shared_ref = buffer_shared_ref?;
    |             ----^^^^^^^^^^^^^^^^^
    |             |
    |             help: remove this `mut`

warning: variable does not need to be mutable
    --> src/widgets/editor_widget/editor_widget.rs:1136:33
     |
1136 | ...                   let mut cursor_set = unpack_or!(buffer.cursors_mut(self.wid), None, "can't get cursor set");
     |                           ----^^^^^^^^^^
     |                           |
     |                           help: remove this `mut`

warning: variable does not need to be mutable
   --> src/widgets/editor_view/editor_view.rs:115:22
    |
115 |     pub fn with_path(mut self, path: SPath) -> Self {
    |                      ----^^^^
    |                      |
    |                      help: remove this `mut`

warning: variable does not need to be mutable
   --> src/widgets/editor_view/editor_view.rs:125:25
    |
125 |     pub fn with_path_op(mut self, path_op: Option<SPath>) -> Self {
    |                         ----^^^^
    |                         |
    |                         help: remove this `mut`

warning: field `widget_id` is never read
  --> src/experiments/focus_group.rs:22:5
   |
21 | pub struct FocusGraphNode<AdditionalData: Clone> {
   |            -------------- field in this struct
22 |     widget_id: WID,
   |     ^^^^^^^^^
   |
   = note: `FocusGraphNode` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis
   = note: `#[warn(dead_code)]` on by default

warning: function `get_full_size` is never used
   --> src/experiments/from_geometry.rs:112:4
    |
112 | fn get_full_size(widgets_and_positions: &Vec<(WID, Rect)>) -> XY {
    |    ^^^^^^^^^^^^^

warning: method `unflatten_index` is never used
  --> src/io/buffer.rs:31:8
   |
31 |     fn unflatten_index(&self, index: usize) -> XY {
   |        ^^^^^^^^^^^^^^^

warning: fields `logger_handle` and `notification_reader_handle` are never read
  --> src/lsp_client/lsp_client.rs:54:5
   |
41 | pub struct LspWrapper {
   |            ---------- fields in this struct
...
54 |     logger_handle: JoinHandle<Result<(), ()>>,
   |     ^^^^^^^^^^^^^
55 |     notification_reader_handle: JoinHandle<Result<(), ()>>,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `read_error_channel` is never read
  --> src/w7e/navcomp_provider_lsp.rs:45:5
   |
41 | pub struct NavCompProviderLsp {
   |            ------------------ field in this struct
...
45 |     read_error_channel: (Sender<LspReadError>, Receiver<LspReadError>),
   |     ^^^^^^^^^^^^^^^^^^

warning: fields `root` and `cargo_file` are never read
  --> src/w7e/rust/handler_cpp:12:5
   |
11 | pub struct RustHandler {
   |            ----------- fields in this struct
12 |     root: SPath,
   |     ^^^^
13 |     cargo_file: cargo_toml::Manifest,
   |     ^^^^^^^^^^

warning: fields `title` and `trigger` are never read
 --> src/widget/action_trigger.rs:5:5
  |
4 | pub struct ActionTrigger<W: Widget> {
  |            ------------- fields in this struct
5 |     title: String,
  |     ^^^^^
6 |     trigger: Box<dyn FnOnce(&W) -> Option<Box<dyn AnyMsg>>>,
  |     ^^^^^^^

warning: field `root_path` is never read
  --> src/widgets/save_file_dialog/save_file_dialog.rs:65:5
   |
49 | pub struct SaveFileDialogWidget {
   |            -------------------- field in this struct
...
65 |     root_path: SPath,
   |     ^^^^^^^^^

warning: fields `on_miss` and `max_width_op` are never read
  --> src/widgets/edit_box.rs:33:5
   |
26 | pub struct EditBoxWidget {
   |            ------------- fields in this struct
...
33 |     on_miss: Option<WidgetAction<EditBoxWidget>>,
   |     ^^^^^^^
...
37 |     max_width_op: Option<u16>,
   |     ^^^^^^^^^^^^

warning: method `event_miss` is never used
   --> src/widgets/edit_box.rs:167:8
    |
167 |     fn event_miss(&self) -> Option<Box<dyn AnyMsg>> {
    |        ^^^^^^^^^^

warning: field `fill_non_free_axis` is never read
  --> src/widgets/with_scroll.rs:25:5
   |
18 | pub struct WithScroll<W: Widget> {
   |            ---------- field in this struct
...
25 |     fill_non_free_axis: bool,
   |     ^^^^^^^^^^^^^^^^^^

warning: field `todo_lable_providers` is never read
   --> src/widgets/editor_widget/editor_widget.rs:171:5
    |
142 | pub struct EditorWidget {
    |            ------------ field in this struct
...
171 |     todo_lable_providers: Vec<LabelsProviderRef>,
    |     ^^^^^^^^^^^^^^^^^^^^

warning: method `get_widget` is never used
   --> src/widgets/editor_widget/editor_widget.rs:127:8
    |
127 |     fn get_widget(&self) -> &dyn Widget {
    |        ^^^^^^^^^^

warning: field `on_miss` is never read
  --> src/widgets/fuzzy_search/fuzzy_search.rs:48:5
   |
37 | pub struct FuzzySearchWidget {
   |            ----------------- field in this struct
...
48 |     on_miss: Option<WidgetAction<Self>>,
   |     ^^^^^^^

warning: field `context_shortcuts` is never read
   --> src/widgets/fuzzy_search/fuzzy_search.rs:183:5
    |
181 | struct ItemIter<'a> {
    |        -------- field in this struct
182 |     providers: &'a Vec<Box<dyn ItemsProvider>>,
183 |     context_shortcuts: &'a Vec<String>,
    |     ^^^^^^^^^^^^^^^^^

warning: field `consider_ignores` is never read
  --> src/widgets/fuzzy_search/fsf_provider.rs:16:5
   |
14 | pub struct FsfProvider {
   |            ----------- field in this struct
15 |     fsf: FsfRef,
16 |     consider_ignores: bool,
   |     ^^^^^^^^^^^^^^^^

warning: constant `CANCEL_LABEL` is never used
  --> src/widgets/generic_dialog/generic_dialog.rs:32:7
   |
32 | const CANCEL_LABEL: &'static str = "Cancel";
   |       ^^^^^^^^^^^^

warning: field `vec` is never read
 --> src/widgets/action_triggers_fuzzy_provicer.rs:8:5
  |
7 | pub struct Actions<W: Widget> {
  |            ------- field in this struct
8 |     vec: Vec<ActionTrigger<W>>,
  |     ^^^

warning: unused `Result` that must be used
  --> src/w7e/navcomp_provider_lsp.rs:96:9
   |
96 |         lock.text_document_did_open(url, file_contents.to_string());
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: this `Result` may be an `Err` variant, which should be handled
   = note: `#[warn(unused_must_use)]` on by default

warning: unused `Result` that must be used
   --> src/w7e/navcomp_provider_lsp.rs:103:9
    |
103 |         lock.text_document_did_change(url, file_contents.to_string());
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled

warning: unused `Result` that must be used
   --> src/w7e/navcomp_provider_lsp.rs:264:9
    |
264 |         lock.text_document_did_close(url);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled

warning: constant `warning_pattern_str` should have an upper case name
  --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:30:7
   |
30 | const warning_pattern_str: &'static str = r"warning: (.+)[\r\n]+ --> (.+):([0-9]+):([0-9]+)[\r\n]+  \|[\r\n]+([0-9]+) \| .+[\r\n]+[^\n]*[\r\n]+";
   |       ^^^^^^^^^^^^^^^^^^^ help: convert the identifier to upper case: `WARNING_PATTERN_STR`
   |
   = note: `#[warn(non_upper_case_globals)]` on by default

warning: unused `Result` that must be used
   --> src/widgets/editor_view/editor_view.rs:174:13
    |
174 |             ff.overwrite_with_stream(&mut buffer.streaming_iterator(), false);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled

warning: `bernardo` (lib) generated 121 warnings (run `cargo fix --lib -p bernardo` to apply 95 suggestions)
warning: unused import: `bernardo::io::buffer::Buffer`
  --> src/bin/reader/main.rs:12:5
   |
12 | use bernardo::io::buffer::Buffer;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `bernardo::io::cell::Cell`
  --> src/bin/reader/main.rs:14:5
   |
14 | use bernardo::io::cell::Cell;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::borrow::BorrowMut`
 --> src/bin/reader/reader_main_widget.rs:1:5
  |
1 | use std::borrow::BorrowMut;
  |     ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Formatter`
 --> src/bin/reader/reader_main_widget.rs:2:23
  |
2 | use std::fmt::{Debug, Formatter};
  |                       ^^^^^^^^^

warning: unused imports: `Key`, `Keycode`
 --> src/bin/reader/reader_main_widget.rs:7:26
  |
7 | use bernardo::io::keys::{Key, Keycode};
  |                          ^^^  ^^^^^^^

warning: unreachable statement
  --> src/bin/reader/main.rs:71:13
   |
70 |             std::process::exit(1);
   |             --------------------- any code following this expression is unreachable
71 |             return;
   |             ^^^^^^^ unreachable statement
   |
   = note: `#[warn(unreachable_code)]` on by default

warning: unused variable: `input_event`
  --> src/bin/reader/reader_main_widget.rs:60:24
   |
60 |     fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
   |                        ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_input_event`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `msg`
  --> src/bin/reader/reader_main_widget.rs:64:26
   |
64 |     fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
   |                          ^^^ help: if this is intentional, prefix it with an underscore: `_msg`

warning: unused variable: `e`
  --> src/bin/reader/main.rs:59:25
   |
59 |                     Err(e) => {}
   |                         ^ help: if this is intentional, prefix it with an underscore: `_e`

warning: variable does not need to be mutable
   --> src/bin/reader/main.rs:108:24
    |
108 |                     Ok(mut ie) => {
    |                        ----^^
    |                        |
    |                        help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` on by default

warning: variant `Close` is never constructed
  --> src/bin/reader/reader_main_widget.rs:38:5
   |
37 | enum ReaderMainWidgetMsg {
   |      ------------------- variant in this enum
38 |     Close,
   |     ^^^^^
   |
   = note: `ReaderMainWidgetMsg` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis
   = note: `#[warn(dead_code)]` on by default

warning: `bernardo` (bin "reader") generated 11 warnings (run `cargo fix --bin "reader"` to apply 9 suggestions)
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
"###;

    const SOME_OUTPUT_2: &str = r###"error[E0425]: cannot find value `line_no_1b` in this scope
  --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:89:44
   |
89 |                 pos: LabelPos::LineAfter { line_no_1b },
   |                                            ^^^^^^^^^^ not found in this scope

warning: unused import: `Metadata`
  --> src/io/buffer_output/buffer_output.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::string::String`
 --> src/io/buffer_output/buffer_output_cells_iter.rs:1:5
  |
1 | use std::string::String;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::io::style::TextStyle`
 --> src/io/buffer_output/buffer_output_cells_iter.rs:5:5
  |
5 | use crate::io::style::TextStyle;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
 --> src/io/buffer_output/buffer_output_cells_iter.rs:6:5
  |
6 | use crate::primitives::rect::Rect;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/io/over_output.rs:8:25
  |
8 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::unpack_or`
  --> src/io/over_output.rs:12:5
   |
12 | use crate::unpack_or;
   |     ^^^^^^^^^^^^^^^^

warning: unused import: `debug`
 --> src/io/sub_output.rs:3:11
  |
3 | use log::{debug, error};
  |           ^^^^^

warning: unused import: `Metadata`
 --> src/io/sub_output.rs:6:25
  |
6 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `Metadata`
  --> src/io/crossterm_output.rs:12:38
   |
12 | use crate::io::output::{FinalOutput, Metadata, Output};
   |                                      ^^^^^^^^

warning: unused import: `crate::cursor::cursor_set::CursorSet`
  --> src/text/buffer_state_fuzz.rs:10:5
   |
10 | use crate::cursor::cursor_set::CursorSet;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `warn`
 --> src/text/contents_and_cursors.rs:3:25
  |
3 | use log::{debug, error, warn};
  |                         ^^^^

warning: unused import: `crate::io::buffer`
  --> src/text/contents_and_cursors.rs:11:5
   |
11 | use crate::io::buffer;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `std::cell::RefCell`
 --> src/tsw/tree_sitter_wrapper.rs:1:5
  |
1 | use std::cell::RefCell;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::rc::Rc`
 --> src/tsw/tree_sitter_wrapper.rs:5:5
  |
5 | use std::rc::Rc;
  |     ^^^^^^^^^^^

warning: unused import: `std::cell::RefCell`
 --> src/tsw/parsing_tuple.rs:1:5
  |
1 | use std::cell::RefCell;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::rc::Rc`
 --> src/tsw/parsing_tuple.rs:3:5
  |
3 | use std::rc::Rc;
  |     ^^^^^^^^^^^

warning: unused import: `std::cell::Cell`
 --> src/w7e/navcomp_provider_lsp.rs:1:5
  |
1 | use std::cell::Cell;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `DocumentSymbolResponse`
 --> src/w7e/navcomp_provider_lsp.rs:8:57
  |
8 | use lsp_types::{CompletionResponse, CompletionTextEdit, DocumentSymbolResponse, Position, SymbolKind};
  |                                                         ^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `NavCompSymbol`, `SymbolPromise`
  --> src/w7e/navcomp_provider_lsp.rs:19:122
   |
19 | use crate::w7e::navcomp_provider::{Completion, CompletionAction, CompletionsPromise, FormattingPromise, NavCompProvider, NavCompSymbol, StupidSubstituteMessage, SymbolContextActionsPromise, SymbolPromise, SymbolType, SymbolUsage, SymbolUsagesPromise};
   |                                                                                                                          ^^^^^^^^^^^^^                                                        ^^^^^^^^^^^^^

warning: unused imports: `Arc`, `RwLock`, `TryLockResult`
 --> src/w7e/workspace.rs:2:17
  |
2 | use std::sync::{Arc, RwLock, TryLockResult};
  |                 ^^^  ^^^^^^  ^^^^^^^^^^^^^

warning: unused imports: `NavCompGroupRef`, `NavCompGroup`
  --> src/w7e/workspace.rs:13:33
   |
13 | use crate::w7e::navcomp_group::{NavCompGroup, NavCompGroupRef};
   |                                 ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/save_file_dialog/save_file_dialog.rs:22:25
   |
22 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/tree_view/tree_view.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/tree_view/tree_view.rs:16:5
   |
16 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/button.rs:8:25
  |
8 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/button.rs:10:5
   |
10 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/edit_box.rs:11:25
   |
11 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/edit_box.rs:14:5
   |
14 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/no_editor.rs:5:25
  |
5 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
 --> src/widgets/no_editor.rs:6:5
  |
6 | use crate::primitives::rect::Rect;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::unpack_or`
 --> src/widgets/no_editor.rs:9:5
  |
9 | use crate::unpack_or;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/with_scroll.rs:5:25
  |
5 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> src/widgets/main_view/main_view.rs:1:5
  |
1 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::io::loading_state::LoadingState`
  --> src/widgets/main_view/main_view.rs:16:5
   |
16 | use crate::io::loading_state::LoadingState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::promise::promise::PromiseState`
  --> src/widgets/main_view/main_view.rs:26:5
   |
26 | use crate::promise::promise::PromiseState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::text::buffer_state::BufferState`
  --> src/widgets/main_view/main_view.rs:27:5
   |
27 | use crate::text::buffer_state::BufferState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::buffer_state_shared_ref::BufferSharedRef`
  --> src/widgets/main_view/main_view.rs:28:5
   |
28 | use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::navcomp_group::NavCompGroupRef`
  --> src/widgets/main_view/main_view.rs:29:5
   |
29 | use crate::w7e::navcomp_group::NavCompGroupRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `either::Either`
 --> src/widgets/main_view/msg.rs:1:5
  |
1 | use either::Either;
  |     ^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::navcomp_provider::SymbolUsagesPromise`
 --> src/widgets/main_view/msg.rs:6:5
  |
6 | use crate::w7e::navcomp_provider::SymbolUsagesPromise;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::RwLockWriteGuard`
 --> src/widgets/editor_widget/editor_widget.rs:2:5
  |
2 | use std::sync::RwLockWriteGuard;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/editor_widget/editor_widget.rs:20:25
   |
20 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::tsw::tree_sitter_wrapper::HighlightItem`
  --> src/widgets/editor_widget/editor_widget.rs:35:5
   |
35 | use crate::tsw::tree_sitter_wrapper::HighlightItem;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `NavCompSymbol`
  --> src/widgets/editor_widget/editor_widget.rs:38:54
   |
38 | use crate::w7e::navcomp_provider::{CompletionAction, NavCompSymbol};
   |                                                      ^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/editor_widget/completion/completion_widget.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/editor_widget/completion/completion_widget.rs:17:5
   |
17 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/editor_widget/context_bar/widget.rs:7:25
  |
7 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/editor_widget/context_bar/widget.rs:12:5
   |
12 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::ops::Range`
 --> src/widgets/editor_widget/label/label.rs:1:5
  |
1 | use std::ops::Range;
  |     ^^^^^^^^^^^^^^^

warning: unused import: `crate::text::text_buffer::TextBuffer`
 --> src/widgets/editor_widget/label/label.rs:6:5
  |
6 | use crate::text::text_buffer::TextBuffer;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::widgets::editor_widget::label::labels_provider::LabelsProvider`
 --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:6:5
  |
6 | use crate::widgets::editor_widget::label::labels_provider::LabelsProvider;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `unpack_or`
 --> src/widgets/fuzzy_search/fuzzy_search.rs:7:13
  |
7 | use crate::{unpack_or, unpack_or_e};
  |             ^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/fuzzy_search/fuzzy_search.rs:13:25
   |
13 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/editor_view/editor_view.rs:10:25
   |
10 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::w7e::handler::NavCompRef`
  --> src/widgets/editor_view/editor_view.rs:24:5
   |
24 | use crate::w7e::handler::NavCompRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::navcomp_group::NavCompGroupRef`
  --> src/widgets/editor_view/editor_view.rs:25:5
   |
25 | use crate::w7e::navcomp_group::NavCompGroupRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::widgets::main_view::main_view::DocumentIdentifier`
  --> src/widgets/editor_view/editor_view.rs:32:5
   |
32 | use crate::widgets::main_view::main_view::DocumentIdentifier;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/list_widget/list_widget.rs:12:25
   |
12 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/list_widget/list_widget.rs:17:5
   |
17 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::rc::Rc`
 --> src/widgets/code_results_view/code_results_widget.rs:3:5
  |
3 | use std::rc::Rc;
  |     ^^^^^^^^^^^

warning: unused import: `std::str::from_utf8`
 --> src/widgets/code_results_view/code_results_widget.rs:4:5
  |
4 | use std::str::from_utf8;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::RwLockWriteGuard`
 --> src/widgets/code_results_view/code_results_widget.rs:5:5
  |
5 | use std::sync::RwLockWriteGuard;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `either::Left`
 --> src/widgets/code_results_view/code_results_widget.rs:7:5
  |
7 | use either::Left;
  |     ^^^^^^^^^^^^

warning: unused import: `regex::internal::Input`
 --> src/widgets/code_results_view/code_results_widget.rs:9:5
  |
9 | use regex::internal::Input;
  |     ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::cursor::cursor::Cursor`
  --> src/widgets/code_results_view/code_results_widget.rs:13:5
   |
13 | use crate::cursor::cursor::Cursor;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::fs::read_error::ReadError`
  --> src/widgets/code_results_view/code_results_widget.rs:16:5
   |
16 | use crate::fs::read_error::ReadError;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
  --> src/widgets/code_results_view/code_results_widget.rs:21:25
   |
21 | use crate::io::output::{Metadata, Output};
   |                         ^^^^^^^^

warning: unused import: `crate::primitives::rect::Rect`
  --> src/widgets/code_results_view/code_results_widget.rs:25:5
   |
25 | use crate::primitives::rect::Rect;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::text::buffer_state::BufferState`
  --> src/widgets/code_results_view/code_results_widget.rs:29:5
   |
29 | use crate::text::buffer_state::BufferState;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::w7e::buffer_state_shared_ref::BufferSharedRef`
  --> src/widgets/code_results_view/code_results_widget.rs:30:5
   |
30 | use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `UpdateResult`
 --> src/widgets/code_results_view/symbol_usage_promise_provider:8:45
  |
8 | use crate::promise::promise::{PromiseState, UpdateResult};
  |                                             ^^^^^^^^^^^^

warning: unused import: `crate::fs::path::SPath`
 --> src/widgets/code_results_view/code_results_msg.rs:3:5
  |
3 | use crate::fs::path::SPath;
  |     ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::widgets::main_view::main_view::DocumentIdentifier`
 --> src/widgets/code_results_view/code_results_msg.rs:5:5
  |
5 | use crate::widgets::main_view::main_view::DocumentIdentifier;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Metadata`
 --> src/widgets/big_list/big_list_widget.rs:6:25
  |
6 | use crate::io::output::{Metadata, Output};
  |                         ^^^^^^^^

warning: unused import: `std::collections::HashMap`
  --> src/cursor/cursor.rs:23:5
   |
23 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `IterMut`, `Iter`
  --> src/cursor/cursor.rs:25:18
   |
25 | use std::slice::{Iter, IterMut};
   |                  ^^^^  ^^^^^^^

warning: unused import: `unicode_segmentation::UnicodeSegmentation`
 --> src/widgets/editor_widget/editor_widget.rs:7:5
  |
7 | use unicode_segmentation::UnicodeSegmentation;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::primitives::printable::Printable`
 --> src/widgets/editor_widget/label/rustc_output_parser_label_provider.rs:4:5
  |
4 | use crate::primitives::printable::Printable;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Promise`
  --> src/widgets/editor_widget/editor_widget.rs:32:31
   |
32 | use crate::promise::promise::{Promise, PromiseState};
   |                               ^^^^^^^

For more information about this error, try `rustc --explain E0425`.
warning: `bernardo` (lib) generated 79 warnings
error: could not compile `bernardo` due to previous error; 79 warnings emitted
"###;
}
