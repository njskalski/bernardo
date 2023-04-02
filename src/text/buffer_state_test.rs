#[cfg(test)]
pub mod tests {
    use crate::primitives::common_edit_msgs::CommonEditMsg;
    use crate::text::buffer_state::BufferState;
    use crate::widget::widget::get_new_widget_id;
    use crate::widgets::main_view::main_view::DocumentIdentifier;

    #[test]
    fn fuzz_1() {
        let mut bf = BufferState::full(None, DocumentIdentifier::new_unique());

        bf.apply_cem(CommonEditMsg::Char('䄀'), get_new_widget_id(), 10, None);
    }
}