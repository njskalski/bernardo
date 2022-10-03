use crate::mocks::mock_output::MockOutput;
use crate::primitives::rect::Rect;

pub struct CompletionInterpreter<'a> {
    rect: Rect,
    mock_output: &'a MockOutput,
}

impl<'a> CompletionInterpreter<'a> {
    pub fn new(rect: Rect, mock_output: &'a MockOutput) -> CompletionInterpreter {
        CompletionInterpreter {
            rect,
            mock_output,
        }
    }
}