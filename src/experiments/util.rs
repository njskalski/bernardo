use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::experiments::focus_group::FocusUpdate;

pub fn default_key_to_focus_update(key_input : InputEvent) -> Option<FocusUpdate> {
    match key_input {
        InputEvent::KeyInput(Key::ArrowLeft) => Some(FocusUpdate::Left),
        InputEvent::KeyInput(Key::ArrowRight) => Some(FocusUpdate::Right),
        InputEvent::KeyInput(Key::ArrowUp) => Some(FocusUpdate::Up),
        InputEvent::KeyInput(Key::ArrowDown) => Some(FocusUpdate::Down),
        InputEvent::KeyInput(Key::Tab) => Some(FocusUpdate::Next),
        // TODO handle shift tab somehow
        _ => None,
    }
}