use log::debug;

use crate::primitives::cursor::Cursor;
use crate::primitives::cursor_set::CursorSet;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::w7e::navcomp_provider::NavCompSymbol;
use crate::widgets::editor_widget::context_bar::context_bar_item::ContextBarItem;
use crate::widgets::editor_widget::editor_widget::EditorState;

/*
I am preemptively moving this code away from EditorWidget, because I expect it to be big
*/

pub fn get_context_options(state: &EditorState,
                           single_cursor: Option<Cursor>,
                           multiple_cursors: &CursorSet,
                           single_stupid_cursor: Option<StupidCursor>,
                           lsp_symbol: Option<&NavCompSymbol>,
                           tree_sitter_symbol: Option<&str>,
) -> Vec<ContextBarItem> {
    let mut results: Vec<ContextBarItem> = Vec::new();

    debug!("hit lsp_symbol, tree_sitter_symbol: {:?} {:?}", &lsp_symbol, &tree_sitter_symbol);

    // WARNING matches are exclusive, with no passthrough, so don't forget about it
    match (state, single_cursor, multiple_cursors, single_stupid_cursor, lsp_symbol, tree_sitter_symbol) {
        (_, Some(_), _, _, _, Some("function")) => {
            results.push(ContextBarItem::GO_TO_DEFINITION);
            results.push(ContextBarItem::SHOW_USAGES);
        }
        _ => {}
    }

    match (state, single_cursor, multiple_cursors, single_stupid_cursor, lsp_symbol, tree_sitter_symbol) {
        (_, _, _, _, Some(_), _) => {
            results.push(ContextBarItem::REFORMAT_FILE);
        }
        _ => {}
    }


    debug!("get_context_options: [{:?}]", &results);

    results
}