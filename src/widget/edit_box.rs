use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Key::Enter;
use crate::widget::edit_box::EditBoxWidgetMsg::Letter;
use crate::widget::widget::{Widget, MsgConstraints, get_new_widget_id, BaseWidget};
use unicode_segmentation::UnicodeSegmentation;
use crate::io::keys::Key;

pub struct EditBoxWidget<ParentMsg: MsgConstraints> {
    id: usize,
    enabled: bool,
    on_hit: Option<fn(&Self) -> Option<ParentMsg>>,
    on_change: Option<fn(&Self) -> Option<ParentMsg>>,
    text: String,
    cursor: usize,
}

impl<ParentMsg: MsgConstraints> EditBoxWidget<ParentMsg> {
    pub fn new() -> Self {
        EditBoxWidget {
            id: get_new_widget_id(),
            cursor: 0,
            enabled: true,
            text: "".into(),
            on_hit: None,
            on_change: None
        }
    }

    pub fn with_on_hit(self, on_hit: fn(&Self) -> Option<ParentMsg>) -> Self {
        EditBoxWidget {
            id: self.id,
            enabled: self.enabled,
            on_hit: Some(on_hit),
            on_change: self.on_change,
            cursor: self.cursor,
            text: self.text,
        }
    }

    pub fn with_on_change(self, on_change: fn(&Self) -> Option<ParentMsg>) -> Self {
        EditBoxWidget {
            id: self.id,
            enabled: self.enabled,
            on_hit: self.on_hit,
            on_change: Some(on_change),
            cursor: self.cursor,
            text: self.text,
        }
    }

    pub fn with_enabled(self, enabled: bool) -> Self {
        EditBoxWidget {
            id : self.id,
            enabled,
            on_hit: self.on_hit,
            cursor: self.cursor,
            text: self.text,
            on_change: self.on_change,
        }
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }
}

impl <ParentMsg: MsgConstraints> BaseWidget for EditBoxWidget<ParentMsg> {
    fn id(&self) -> usize {
        self.id
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditBoxWidgetMsg {
    Hit,
    Letter(char),
}

impl<ParentMsg : MsgConstraints> Widget<ParentMsg> for EditBoxWidget<ParentMsg> {
    type LocalMsg = EditBoxWidgetMsg;

    fn update(&mut self, msg: EditBoxWidgetMsg) -> Option<ParentMsg> {
        match msg {
            EditBoxWidgetMsg::Hit => {
                if self.on_hit.is_none() {
                    None
                } else {
                    self.on_hit.unwrap()(&self)
                }
            }
            EditBoxWidgetMsg::Letter(ch) => {
                let mut iter = self.text.graphemes(true);
                let mut new_text = String::new();

                for _ in 0..self.cursor {
                    new_text += iter.as_str();
                    iter.next();
                }

                new_text += ch.to_string().as_str(); //TODO: make this conversion better?

                for _ in self.cursor..self.text.len() {
                    new_text += iter.as_str();
                    iter.next();
                }

                self.text = new_text;

                if self.on_change.is_some() {
                    self.on_change.unwrap()(self)
                } else {
                    None
                }
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
            KeyInput(Key::Letter(ch)) => Some(EditBoxWidgetMsg::Letter(ch)),
            _ => None,
        }
    }
}
