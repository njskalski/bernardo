use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Key::Enter;
use crate::widget::widget::Widget;

pub struct ButtonWidget<ParentMsg: MsgConstraints> {
    enabled: bool,
    on_hit: Option<fn(&Self) -> Option<ParentMsg>>,
}

impl<ParentMsg: MsgConstraints> ButtonWidget<ParentMsg> {
    fn new() -> Self {
        ButtonWidget {
            enabled: true,
            on_hit: None,
        }
    }

    fn with_on_hit(self, on_hit: fn(&Self) -> Option<ParentMsg>) -> Self {
        ButtonWidget {
            enabled: self.enabled,
            on_hit: Some(on_hit),
        }
    }

    fn with_enabled(self, enabled: bool) -> Self {
        ButtonWidget {
            enabled,
            on_hit: self.on_hit,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum ButtonWidgetMsg {
    Hit,
    // Focus,
    // LostFocus
}

impl Widget<ParentMsg> for ButtonWidget<ParentMsg> {
    type LocalMsg = ButtonWidgetMsg;

    fn update(&mut self, msg: LocalMsg) -> Option<ParentMsg> {
        match msg {
            ButtonWidgetMsg::Hit => {
                if self.on_hit.is_none() {
                    None
                } else {
                    self.on_hit.unwrap()(&self)
                }
            }
            _ => None,
        }
    }

    fn focusable(&self) -> bool {
        self.enabled
    }

    fn on_input(&self, input_event: InputEvent) -> Option<ButtonWidgetMsg> {
        debug_assert!(
            self.enabled,
            "ButtonWidget: received input to disabled component!"
        );

        match input_event {
            KeyInput(Enter) => Some(ButtonWidgetMsg::Hit),
            _ => None,
        }
    }
}
