use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

pub struct EditorWidget {
    id: WID,
    showing_save: bool,
    text: String,
}

impl EditorWidget {
    pub fn new() -> Self {
        EditorWidget {
            id: get_new_widget_id(),
            showing_save: false,
            text: "".to_owned(),
        }
    }

    pub fn with_text(self, text: String) -> Self {
        EditorWidget {
            text,
            ..self
        }
    }
}

impl Widget for EditorWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "EditorWidget"
    }

    fn min_size(&self) -> XY {
        // completely arbitrary
        XY::new(48, 24)
    }

    fn layout(&mut self, max_size: XY) -> XY {
        max_size
    }

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn get_focused(&self) -> &dyn Widget {
        if self.showing_save {}
        todo!()
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        todo!()
    }

    fn render(&self, _theme: &Theme, _focused: bool, _output: &mut dyn Output) {
        todo!()
    }
}