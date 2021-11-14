use std::thread;

use crossbeam_channel::Receiver;
use log::debug;

use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::input_source::InputSource;

struct CrosstermInput {
    receiver: Receiver<InputEvent>,
}

impl CrosstermInput {
    pub fn new() -> Self {
        let (send, recv) = crossbeam_channel::unbounded::<InputEvent>();

        thread::spawn(move || {
            loop {
                let event = crossterm::event::read();
                match event {
                    Ok(raw_event) => {
                        raw_event
                    }
                    Err(err) => {
                        debug!("received error {:?}, closing crossterm input.", err);
                        break;
                    }
                }
            }
        });

        CrosstermInput {
            receiver: recv
        }
    }
}

impl Input for CrosstermInput {
    fn source(&self) -> &InputSource {
        &self.receiver
    }
}