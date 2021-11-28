use crate::io::keys::Key;


#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Tick,
    // state might have changed, update and redraw if necessary
    KeyInput(Key),
}
