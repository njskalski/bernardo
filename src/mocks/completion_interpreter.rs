use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::rect::Rect;

pub struct CompletionInterpreter<'a> {
    rect: Rect,
    output: &'a MetaOutputFrame,
}

impl<'a> CompletionInterpreter<'a> {
    pub fn new(rect: Rect, output: &'a MetaOutputFrame) -> CompletionInterpreter {
        CompletionInterpreter {
            rect,
            output,
        }
    }
}