use crossbeam_channel::{Receiver, Sender};
use which::Path;

use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::input_source::InputSource;

struct MockInput {
    sender: Sender<InputEvent>,
    receiver: Receiver<InputEvent>,
}

impl Input for MockInput {
    fn source(&self) -> &InputSource {
        &self.receiver
    }
}

impl MockInput {
    pub fn send_event(&self, ie: InputEvent) {
        self.sender.send(ie).unwrap()
    }
}