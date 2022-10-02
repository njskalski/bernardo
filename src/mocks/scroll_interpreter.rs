use crate::mocks::mock_output::MockOutput;
use crate::primitives::rect::Rect;

pub struct ScrollInterpreter<'a> {
    rect: Rect,
    mock_output: &'a MockOutput,
}

impl<'a> ScrollInterpreter<'a> {
    pub fn new(rect: Rect, mock_output: &'a MockOutput) -> ScrollInterpreter {
        ScrollInterpreter {
            rect,
            mock_output,
        }
    }

    pub fn lowest_number(&self) -> Option<usize> {
        self.mock_output.frontbuffer().lines_iter().with_rect(self.rect).next().map(|line| {
            line.parse::<usize>().ok()
        }).flatten()
    }

    pub fn highest_number(&self) -> Option<usize> {
        self.mock_output.frontbuffer().lines_iter().with_rect(self.rect).last().map(|line| {
            line.parse::<usize>().ok()
        }).flatten()
    }
}