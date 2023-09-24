use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

pub struct MockWidget {
    id: WID,
    pub full_size: XY,
    pub last_layout: Option<Screenspace>,
    pub size_policy: SizePolicy,
}

impl MockWidget {
    pub fn new(full_size: XY) -> Self {
        MockWidget {
            id: get_new_widget_id(),
            full_size,
            last_layout: None,
            size_policy: SizePolicy::SELF_DETERMINED,
        }
    }

    pub fn with_size_policy(self, size_policy: SizePolicy) -> Self {
        Self {
            size_policy,
            ..self
        }
    }
}

impl Widget for MockWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str where Self: Sized {
        "MockWidget"
    }

    fn typename(&self) -> &'static str {
        "MockWidget"
    }

    fn full_size(&self) -> XY {
        self.full_size
    }

    fn size_policy(&self) -> SizePolicy {
        self.size_policy
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.last_layout = Some(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {}
}

