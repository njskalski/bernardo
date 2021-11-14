use crate::experiments::focus_group::FocusUpdate;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;

pub fn default_key_to_focus_update(key_input: InputEvent) -> Option<FocusUpdate> {
    match key_input {
        InputEvent::KeyInput(Keycode::ArrowLeft) => Some(FocusUpdate::Left),
        InputEvent::KeyInput(Keycode::ArrowRight) => Some(FocusUpdate::Right),
        InputEvent::KeyInput(Keycode::ArrowUp) => Some(FocusUpdate::Up),
        InputEvent::KeyInput(Keycode::ArrowDown) => Some(FocusUpdate::Down),
        InputEvent::KeyInput(Keycode::Tab) => Some(FocusUpdate::Next),
        // TODO handle shift tab somehow
        _ => None,
    }
}
