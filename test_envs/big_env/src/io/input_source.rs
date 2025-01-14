use crate::io::input_event::InputEvent;

pub type InputSource = crossbeam_channel::Receiver<InputEvent>;
