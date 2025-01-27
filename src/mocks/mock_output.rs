use std::fmt::{Debug, Formatter};
use std::io::Error;

use crate::config::theme::Theme;
use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::output::{FinalOutput, Metadata, Output};
use crate::io::style::TextStyle;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crossbeam_channel::{Receiver, SendError, Sender};
use log::{debug, error};

pub struct MockOutput {
    buffer_0: BufferOutput,
    buffer_1: BufferOutput,
    which_front: bool,
    theme: Theme,

    sender: Sender<MetaOutputFrame>,

    metadata: Vec<Metadata>,
}

impl MockOutput {
    pub fn new(size: XY, bounded: bool, theme: Theme) -> (MockOutput, Receiver<MetaOutputFrame>) {
        let (sender, receiver) = if bounded {
            crossbeam_channel::bounded::<MetaOutputFrame>(1)
        } else {
            crossbeam_channel::unbounded::<MetaOutputFrame>()
        };

        (
            MockOutput {
                buffer_0: BufferOutput::new(size),
                buffer_1: BufferOutput::new(size),
                which_front: false,
                theme,
                sender,
                metadata: Vec::new(),
            },
            receiver,
        )
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
        write!(f, "[MockBuffer front: {} sc: {:?}]", self.which_front, self.buffer_0.size())
    }
}

impl SizedXY for MockOutput {
    fn size(&self) -> XY {
        self.buffer_0.size()
    }
}

impl Output for MockOutput {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        self.backbuffer_mut().print_at(pos, style, text)
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.backbuffer_mut().clear()?;
        self.metadata.clear();
        Ok(())
    }

    fn visible_rect(&self) -> Rect {
        let res = Rect::from_zero(self.buffer_0.size());
        debug_assert!(res.lower_right() <= self.size());
        res
    }

    #[cfg(any(test, feature = "fuzztest"))]
    fn emit_metadata(&mut self, meta: Metadata) {
        debug!("metadata called with {:?}", meta);
        self.metadata.push(meta)
    }
}

impl FinalOutput for MockOutput {
    fn end_frame(&mut self) -> Result<(), Error> {
        self.which_front = !self.which_front;

        debug!("MockOutput.end_frame - sending. Metadata: {:?}", self.metadata);

        let msg = MetaOutputFrame {
            buffer: self.frontbuffer().clone(),
            metadata: self.metadata.clone(),
            theme: self.theme.clone(),
        };

        if let Err(e) = self.sender.send(msg) {
            error!(
                "failed sending Mock Output frame with Metadata {:?}, because {:?}",
                self.metadata, e
            );
        };

        Ok(())
    }

    fn get_front_buffer(&self) -> &BufferOutput {
        self.frontbuffer()
    }
}
