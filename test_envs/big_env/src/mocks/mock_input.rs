use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::input_source::InputSource;

pub struct MockInput {
    receiver: Receiver<InputEvent>,
}

impl Input for MockInput {
    fn source(&self) -> &InputSource {
        &self.receiver
    }
}

impl MockInput {
    pub fn new() -> (MockInput, Sender<InputEvent>) {
        let (sender, receiver) = unbounded::<InputEvent>();

        (MockInput { receiver }, sender)
    }
}
