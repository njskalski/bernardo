#[cfg(test)]
pub mod tests {
    use crate::primitives::common_edit_msgs::CommonEditMsg;
    use crate::text::buffer_state::BufferState;

    #[test]
    fn fuzz_1() {
        let mut bf = BufferState::full(None);

        bf.apply_cem(CommonEditMsg::Char('䄀'), 10, None);
    }
}