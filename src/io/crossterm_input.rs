use std::thread;

use crossbeam_channel::Receiver;
use crossterm::event::Event;
use log::{debug, error, warn};

use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::input_source::InputSource;
use crate::io::keys::Key;

pub struct CrosstermInput {
    receiver: Receiver<InputEvent>,
}

impl CrosstermInput {
    pub fn new() -> Self {
        let (send, recv) = crossbeam_channel::unbounded::<InputEvent>();

        thread::spawn(move || {
            loop {
                let event = crossterm::event::read();
                match event {
                    Err(err) => {
                        error!("received error {:?}, closing crossterm input.", err);
                        break;
                    }
                    Ok(raw_event) => {
                        let processed_event: Option<InputEvent> = match raw_event {
                            Event::Key(ckey) => {
                                let key: Key = ckey.into();
                                Some(InputEvent::KeyInput(key))
                            }
                            Event::Mouse(_) =>
                                None,
                            Event::Resize(_, _) => {
                                None // TODO
                            }
                        };

                        // debug!("got {:?}", processed_event);

                        match processed_event {
                            None => continue,
                            Some(event) => {
                                match send.send(event) {
                                    Ok(_) => continue,
                                    Err(err) => {
                                        warn!("failed sending event {:?} because {}", event, err);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                };
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