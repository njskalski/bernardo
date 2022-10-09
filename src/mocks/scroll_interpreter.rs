use crate::mocks::meta_frame::MetaOutputFrame;
use crate::primitives::rect::Rect;

pub struct ScrollInterpreter<'a> {
    rect: Rect,
    output: &'a MetaOutputFrame,
}

impl<'a> ScrollInterpreter<'a> {
    pub fn new(rect: Rect, mock_output: &'a MetaOutputFrame) -> ScrollInterpreter {
        ScrollInterpreter {
            rect,
            output: mock_output,
        }
    }

    pub fn lowest_number(&self) -> Option<usize> {
        self.output.buffer.lines_iter().with_rect(self.rect).next().map(|line| {
            line.trim().parse::<usize>().ok()
        }).flatten()
    }

    pub fn highest_number(&self) -> Option<usize> {
        self.output.buffer.lines_iter().with_rect(self.rect).last().map(|line| {
            line.parse::<usize>().ok()
        }).flatten()
    }
}