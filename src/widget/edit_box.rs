use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Key::Enter;
use crate::widget::edit_box::EditBoxWidgetMsg::Letter;
use crate::widget::widget::Widget;
use unicode_segmentation::UnicodeSegmentation;

pub struct EditBoxWidget<ParentMsg: MsgConstraints> {
    enabled: bool,
    on_hit: Option<fn(&Self) -> Option<ParentMsg>>,
    on_change: Option<fn(&Self) -> Option<ParentMsg>>,
    text: String,
    cursor: usize,
}

impl<ParentMsg: MsgConstraints> EditBoxWidget<ParentMsg> {
    fn new() -> Self {
        EditBoxWidget {
            cursor: 0,
            enabled: true,
            text: "".into(),
            on_hit: None,
            on_change: None,
        }
    }

    fn with_on_hit(self, on_hit: fn(&Self) -> Option<ParentMsg>) -> Self {
        EditBoxWidget {
            enabled: self.enabled,
            on_hit: Some(on_hit),
            on_change: self.on_change,
            cursor: self.cursor,
            text: self.text,
        }
    }

    fn with_enabled(self, enabled: bool) -> Self {
        EditBoxWidget {
            enabled,
            on_hit: self.on_hit,
            cursor: self.cursor,
            text: self.text,
            on_change: self.on_change,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum EditBoxWidgetMsg {
    Hit,
    Letter(char),
}

impl Widget<ParentMsg> for EditBoxWidget<ParentMsg> {
    type LocalMsg = EditBoxWidgetMsg;

    fn update(&mut self, msg: LocalMsg) -> Option<ParentMsg> {
        match msg {
            ButtonWidgetMsg::Hit => {
                if self.on_hit.is_none() {
                    None
                } else {
                    self.on_hit.unwrap()(&self)
                }
            }
            EditBoxWidgetMsg::Letter(ch) => {
                let mut iter = self.text.unicode_words().into_iter();
                let mut new_text = String::new();

                for _ in 0..self.cursor {
                    new_text += *iter;
                    iter.next();
                }

                new_text += ch.into();

                for _ in self.cursor..self.text.len() {
                    new_text += *iter;
                    iter.next()
                }

                self.text = new_text
            }
            _ => None,
        }
    }

    fn focusable(&self) -> bool {
        self.enabled
    }

    fn on_input(&self, input_event: InputEvent) -> Option<EditBoxWidgetMsg> {
        debug_assert!(
            self.enabled,
            "EditBoxWidgetMsg: received input to disabled component!"
        );

        match input_event {
            KeyInput(Enter) => Some(EditBoxWidgetMsg::Hit),
            KeyInput(Letter(ch)) => Some(EditBoxWidgetMsg::Letter(ch)),
            _ => None,
        }
    }
}
