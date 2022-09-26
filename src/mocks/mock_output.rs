use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::Error;

use crossbeam_channel::{Receiver, Sender};

use crate::io::buffer_output::BufferOutput;
use crate::io::output::{FinalOutput, Output};
use crate::io::style::TextStyle;
use crate::mocks::extended_info::ExtendedInfo;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::widget::WID;

pub type ExtInfo = HashMap<(&'static str, WID), ExtendedInfo>;

pub struct MockOutput {
    buffer_0: BufferOutput,
    buffer_1: BufferOutput,
    which_front: bool,

    sender: Sender<BufferOutput>,
}

impl MockOutput {
    pub fn new(size: XY, bounded: bool) -> (MockOutput, Receiver<BufferOutput>) {
        let (sender, receiver) = if bounded {
            crossbeam_channel::bounded::<BufferOutput>(1)
        } else {
            crossbeam_channel::unbounded::<BufferOutput>()
        };

        (MockOutput {
            buffer_0: BufferOutput::new(size),
            buffer_1: BufferOutput::new(size),
            which_front: false,
            sender,
        }, receiver)
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

    fn get_final_position(&self, local_pos: XY) -> Option<XY> {
        if local_pos < self.buffer_0.size() {
            Some(local_pos)
        } else {
            None
        }
    }
}

impl FinalOutput for MockOutput {
    fn end_frame(&mut self) -> Result<(), Error> {
        self.which_front = !self.which_front;

        let msg = self.frontbuffer().clone();
        self.sender.send(msg).unwrap();

        Ok(())
    }

    fn get_front_buffer(&self) -> &BufferOutput {
        self.frontbuffer()
    }
}