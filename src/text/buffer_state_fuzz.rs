use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::text::buffer_state::BufferState;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};

impl<'a> Arbitrary<'a> for BufferState {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let document = DocumentIdentifier::new_unique();
        let text = String::arbitrary(u)?;

        Ok(BufferState::full(None, document, None, None).with_text(text.clone()))
    }
}

pub fn fuzz_call(text: String, msgs: Vec<CommonEditMsg>) {
    let docid = DocumentIdentifier::new_unique();
    let mut bf = BufferState::full(None, docid, None, None).with_text(text);
    bf.initialize_for_widget(1, None);

    for msg in msgs {
        bf.apply_common_edit_message(msg, 1, 3, None, true);
    }
}
