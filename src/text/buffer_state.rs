use ropey::Rope;
use crate::text::buffer::Buffer;

pub struct BufferState {
    text: Rope,
    prev: Option<Box<BufferState>>,
    next: Option<Box<BufferState>>,
}

impl BufferState {
    pub fn new() -> BufferState {
        BufferState {
            text: Rope::new(),
            prev: None,
            next: None,
        }
    }

    pub fn with_text(self, text: &str) -> BufferState {
        BufferState {
            text: Rope::from_str(text),
            ..self
        }
    }
}

impl Buffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text.len_lines()
    }
    fn lines(&self) -> Box<dyn std::iter::Iterator<Item=&str> + '_> {
        Box::new(self.text.lines().map(|line| line.as_str().unwrap()))
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    fn char_to_line(&self, char_idx: usize) -> usize {
        self.text.char_to_line(char_idx)
    }

    fn line_to_char(&self, line_idx: usize) -> usize {
        self.text.line_to_char(line_idx)
    }
}