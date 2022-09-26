use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

pub struct NoEditorWidget {
    wid: WID,
    text_pos: XY,
}

impl NoEditorWidget {
    pub const NO_EDIT_TEXT: &'static str = "no editor loaded.";
}

impl Default for NoEditorWidget {
    fn default() -> Self {
        NoEditorWidget {
            wid: get_new_widget_id(),
            text_pos: XY::ZERO,
        }
    }
}

impl Widget for NoEditorWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "no_editor_widget"
    }

    fn min_size(&self) -> XY {
        XY::new(Self::NO_EDIT_TEXT.len() as u16, 3)
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        let mut x = 0;
        if sc.visible_hint().size.x >= Self::NO_EDIT_TEXT.len() as u16 {
            x = (sc.visible_hint().size.x - Self::NO_EDIT_TEXT.len() as u16) / 2;
        };

        let y = sc.visible_hint().size.y / 2;

        self.text_pos = XY::new(x, y);

        sc.visible_hint().size
    }

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> { None }

    fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        // fill_background(theme.default_background(focused), output);

        output.print_at(self.text_pos,
                        theme.default_text(focused),
                        Self::NO_EDIT_TEXT,
                        self.ext(),
        );
    }
}