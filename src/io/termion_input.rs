use crate::io::input_event::InputEvent;
use crate::io::input_source::InputSource;
use crossbeam_channel::Receiver;
use std::io::{Error, Read};

use log::debug;

use std::thread;

use termion::input::{TermRead, TermReadEventsAndRaw};
use termion::raw::IntoRawMode;
use termion::AsyncReader;
// use termion::event::Event::Key as TKey;
use crate::io::keys::Key;
use crate::io::keys::Key::Letter;
use termion::event::Event;

pub struct TermionInput {
    receiver: Receiver<InputEvent>,
}

impl TermionInput {
    pub fn new<R: 'static + Read + Send>(mut stdin: R) -> Self {
        // TODO buffer-size?
        let (mut sender, mut receiver) = crossbeam_channel::bounded::<InputEvent>(1);

        thread::spawn(move || {
            debug!("Starting termion input thread.");

            for c in stdin.events_and_raw() {
                match c {
                    Ok((event, data)) => match event {
                        Event::Key(key) => {
                            let my_key: Key = key.into();
                            sender.send(InputEvent::KeyInput(my_key));
                        }
                        Event::Mouse(me) => {
                            debug!("Ignoring mouse event {:?}", me);
                        }
                        Event::Unsupported(ue) => {
                            debug!("Ignoring unsupported event {:?}", ue);
                        }
                    },
                    Err(e) => {
                        debug!("Error reading input from Termion : {:?}", e);
                        break;
                    }
                }
            }
        });

        TermionInput { receiver }
    }

    pub fn source(&self) -> &InputSource {
        &self.receiver
    }
}
