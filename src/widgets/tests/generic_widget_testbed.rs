use crossbeam_channel::Receiver;
use lazy_static::lazy_static;

use crate::config::theme::Theme;
use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::{FinalOutput, Output};
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_navcomp_provider::MockNavCompProviderPilot;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;

pub struct GenericWidgetTestbed<W: Widget, AdditionalData = ()> {
    pub widget: W,
    pub additional_data: AdditionalData,
    pub size: XY,
    pub providers: Providers,
    pub last_frame: Option<MetaOutputFrame>,
    pub mock_navcomp_pilot: Option<MockNavCompProviderPilot>,
    pub output: MockOutput,
    pub recv: Receiver<MetaOutputFrame>,
    pub last_msg: Option<Box<dyn AnyMsg>>,
}

lazy_static! {
    static ref DEFAULT_THEME : Theme = Theme::default();
}

impl<W: Widget, AdditionalData> GenericWidgetTestbed<W, AdditionalData> {
    pub fn next_frame(&mut self) {
        self.output.clear().unwrap();
        self.widget.prelayout();
        self.widget.layout(Screenspace::full_output(self.size));
        let theme = self.theme().clone();
        self.widget.render(&theme, true, &mut self.output);

        self.output.end_frame().unwrap();

        let frame = self.recv.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn theme(&self) -> &Theme {
        self.providers.theme()
    }
    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }

    pub fn send_input(&mut self, input: InputEvent) {
        self.last_msg = self.widget.act_on(input).1;
        self.next_frame();
    }

    pub fn widget(&self) -> &W {
        &self.widget
    }
    pub fn widget_mut(&mut self) -> &mut W {
        &mut self.widget
    }

    pub fn push_input(&mut self, input: InputEvent) {
        let (_, last_msg) = self.widget.act_on(input);
        self.last_msg = last_msg;
        self.next_frame();
    }

    pub fn push_text(&mut self, text: &str) {
        for char in text.chars() {
            self.push_input(Keycode::Char(char).to_key().to_input_event())
        }
    }
}

impl<W: Widget, AdditionalData> WithWaitFor for GenericWidgetTestbed<W, AdditionalData> {
    fn is_frame_based_wait(&self) -> bool {
        false
    }

    fn last_frame(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    fn set_last_frame(&mut self, meta_output_frame: MetaOutputFrame) {
        self.last_frame = Some(meta_output_frame);
    }

    fn output_receiver(&self) -> &Receiver<MetaOutputFrame> {
        &self.recv
    }
}
