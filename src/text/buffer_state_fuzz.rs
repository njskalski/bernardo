use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};

use crate::text::buffer_state::{BufferState, BufferType};
use crate::widgets::main_view::main_view::DocumentIdentifier;

impl<'a> Arbitrary<'a> for BufferState {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let subtype = u.arbitrary::<BufferType>()?;
        let document = u.arbitrary::<DocumentIdentifier>()?;

        let mut text = u.arbitrary::<String>()?;
        let bf = match subtype {
            BufferType::Full => BufferState::full(None, document).with_text(text.clone()),
            BufferType::SingleLine => {
                text = text.replace('\n', "");
                BufferState::simplified_single_line().with_text(text.clone())
            }
        };

        Ok(bf)
    }
}
