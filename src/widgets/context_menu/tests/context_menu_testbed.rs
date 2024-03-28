use crossbeam_channel::Receiver;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::{FinalOutput, Output};
use crate::mocks::context_menu_interpreter::ContextMenuInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::mock_providers_builder::MockProvidersBuilder;
use crate::mocks::mock_tree_item::MockTreeItem;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;
use crate::widgets::context_menu::widget::ContextMenuWidget;

#[derive(Debug, Eq, PartialEq)]
pub enum ContextMenuTestMsg {
    Text(String),
}

impl AnyMsg for ContextMenuTestMsg {}

pub struct ContextMenuTestbed {
    context_menu: ContextMenuWidget<String, MockTreeItem>,
    size: XY,
    config: ConfigRef,
    theme: Theme,
    last_frame: Option<MetaOutputFrame>,
    last_msg: Option<Box<dyn AnyMsg>>,

    output: MockOutput,
    recv: Receiver<MetaOutputFrame>,
}

impl ContextMenuTestbed {
    pub fn new(mock_data_set: MockTreeItem) -> Self {
        let size = XY::new(30, 20);
        let providers = MockProvidersBuilder::new().build().providers;

        let theme: Theme = Default::default();

        let (output, recv) = MockOutput::new(size, false, theme.clone());

        ContextMenuTestbed {
            context_menu: ContextMenuWidget::new(providers, mock_data_set),
            size,
            config: Default::default(),
            theme,
            last_frame: None,
            last_msg: None,
            output,
            recv,
        }
    }
    pub fn context_menu(&self) -> Option<ContextMenuInterpreter<'_>> {
        self.last_frame.as_ref().map(|frame| frame.get_context_menus().next()).flatten()
    }

    pub fn next_frame(&mut self) {
        self.output.clear().unwrap();
        self.context_menu.prelayout();
        self.context_menu.layout(Screenspace::full_output(self.size));
        self.context_menu.render(&self.theme, true, &mut self.output);

        self.output.end_frame().unwrap();

        let frame = self.recv.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        let (_, last_msg) = self.context_menu.act_on(input);
        self.last_msg = last_msg;
        self.next_frame();
    }

    pub fn push_text(&mut self, text: &str) {
        for char in text.chars() {
            self.push_input(Keycode::Char(char).to_key().to_input_event())
        }
    }

    pub fn has_items<'a, I: Iterator<Item = &'a str>>(&self, items: I) -> bool {
        for item_label in items {
            if self
                .context_menu()
                .unwrap()
                .tree_view()
                .items()
                .iter()
                .find(|item| item.label.as_str() == item_label)
                .is_none()
            {
                return false;
            }
        }

        true
    }

    pub fn has_none_of_items<'a, I: Iterator<Item = &'a str>>(&self, items: I) -> bool {
        for item_label in items {
            if self
                .context_menu()
                .unwrap()
                .tree_view()
                .items()
                .iter()
                .find(|item| item.label.as_str() == item_label)
                .is_some()
            {
                return false;
            }
        }

        true
    }
}

impl WithWaitFor for ContextMenuTestbed {
    fn is_frame_based_wait(&self) -> bool {
        false
    }

    fn last_frame(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    fn set_last_frame(&mut self, meta_output_frame: MetaOutputFrame) {
        self.last_frame = Some(meta_output_frame)
    }

    fn output_receiver(&self) -> &Receiver<MetaOutputFrame> {
        &self.recv
    }
}
