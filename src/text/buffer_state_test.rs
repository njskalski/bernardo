#[cfg(test)]
pub mod tests {
    use crate::text::buffer_state::BufferState;
    use crate::widget::widget::get_new_widget_id;
    use crate::widgets::main_view::main_view::DocumentIdentifier;
    use crate::{primitives::common_edit_msgs::CommonEditMsg, text::buffer_state::BufferType};
    // use crate::text::buffer_type::BufferType;
    use crate::text::contents_and_cursors::ContentsAndCursors;
    // use crate::text::document_identifier::DocumentIdentifier;
    // use crate::text::lang_id::LangId;
    // use crate::text::tree_sitter_wrapper::TreeSitterWrapper;
    use crate::experiments::clipboard::ClipboardRef;
    // use crate::io::keys::WID;
    use std::sync::{mpsc, Arc};

    #[test]
    fn fuzz_1() {
        let mut bf = BufferState::full(None, DocumentIdentifier::new_unique(), None);

        bf.apply_common_edit_message(CommonEditMsg::Char('䄀'), get_new_widget_id(), 10, None, false);
    }
}
