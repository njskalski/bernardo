use std::io::{Read};
use std::thread;

use crossbeam_channel::Receiver;
use log::debug;

use termion::event::Event;
use termion::input::{TermReadEventsAndRaw};


use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::input_source::InputSource;
// use termion::event::Event::Key as TKey;
use crate::io::keys::{Key};

pub struct TermionInput {
    receiver: Receiver<InputEvent>,
}

impl TermionInput {
    pub fn new<R: 'static + Read + Send>(stdin: R) -> Self {
        // TODO buffer-size?
        let (sender, receiver) = crossbeam_channel::bounded::<InputEvent>(1);

        thread::spawn(move || {
            debug!("Starting termion input thread.");

            for c in stdin.events_and_raw() {
                match c {
                    Ok((event, _data)) => match event {
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

}

impl Input for TermionInput {
    fn source(&self) -> &InputSource {
        &self.receiver
    }
}