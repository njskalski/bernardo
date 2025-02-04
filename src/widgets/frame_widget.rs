use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::border::BorderStyle;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use log::error;

pub struct FrameWidget {
    wid: WID,
    style: BorderStyle,
    label_op: Option<String>,

    last_rect_inclusive: Option<Rect>,
}

impl FrameWidget {
    pub const TYPENAME: &'static str = "frame_widget";

    pub fn new(style: BorderStyle, label_op: Option<String>) -> Self {
        FrameWidget {
            wid: get_new_widget_id(),
            style,
            label_op,
            last_rect_inclusive: None,
        }
    }
}

impl Widget for FrameWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }
    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }
    fn size_policy(&self) -> SizePolicy {
        SizePolicy::SELF_DETERMINED
    }

    fn full_size(&self) -> XY {
        if let Some(last_rect_inclusive) = self.last_rect_inclusive {
            last_rect_inclusive.lower_right() + XY::new(1, 1)
        } else {
            error!("full_size before layout");
            XY::new(2, 2)
        }
    }

    fn layout(&mut self, screenspace: Screenspace) {
        if screenspace.output_size() >= XY::new(2, 2) {
            let rect = Rect::new(XY::ZERO, screenspace.output_size() - XY::new(1, 1));
            self.last_rect_inclusive = Some(rect);
        } else {
            self.last_rect_inclusive = None;
        }
    }

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        // DO NOT ADD CLEAR HERE

        if let Some(rect) = &self.last_rect_inclusive {
            let text_style = theme.default_text(focused);

            self.style
                .draw_full_rect(text_style, output, rect.clone(), self.label_op.as_deref())
        } else {
            error!("render before layout");
        }
    }

    fn is_focusable(&self) -> bool {
        false
    }
}
