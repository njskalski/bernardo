use std::fmt::{Debug, Formatter};
use std::io::Error;

use crate::io::buffer::Buffer;
use crate::io::buffer_output::BufferOutput;
use crate::io::output::{FinalOutput, Output};
use crate::io::style::TextStyle;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;

pub struct MockOutput {
    buffer_0: BufferOutput,
    buffer_1: BufferOutput,
    which_front: bool,
}

impl MockOutput {
    pub fn new(size: XY) -> MockOutput {
        MockOutput {
            buffer_0: BufferOutput::new(size),
            buffer_1: BufferOutput::new(size),
            which_front: false,
        }
    }

    pub fn frontbuffer(&self) -> &BufferOutput {
        if self.which_front == false {
            &self.buffer_0
        } else {
            &self.buffer_1
        }
    }

    pub fn frontbuffer_mut(&mut self) -> &mut BufferOutput {
        if self.which_front == false {
            &mut self.buffer_0
        } else {
            &mut self.buffer_1
        }
    }

    pub fn backbuffer(&self) -> &BufferOutput {
        if self.which_front {
            &self.buffer_0
        } else {
            &self.buffer_1
        }
    }

    pub fn backbuffer_mut(&mut self) -> &mut BufferOutput {
        if self.which_front {
            &mut self.buffer_0
        } else {
            &mut self.buffer_1
        }
    }
}

impl Debug for MockOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[MockBuffer front: {} sc: {:?}]", self.which_front, self.buffer_0.size_constraint())
    }
}

impl Output for MockOutput {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        self.backbuffer_mut().print_at(pos, style, text)
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.backbuffer_mut().clear()
    }

    fn size_constraint(&self) -> SizeConstraint {
        self.buffer_0.size_constraint()
    }
}

impl FinalOutput for MockOutput {
    fn end_frame(&mut self) -> Result<(), Error> {
        self.which_front = !self.which_front;
        Ok(())
    }

    fn get_front_buffer(&self) -> &BufferOutput {
        self.frontbuffer()
    }
}