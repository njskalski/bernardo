use std::rc::Rc;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::screen_shot::screenshot;
use crate::gladius::paradigm::recursive_treat_views;
use crate::io::input_event::InputEvent;
use crate::io::output::FinalOutput;
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_navcomp_provider::MockNavCompProviderPilot;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::list_widget::list_widget::ListWidget;
use crate::widgets::list_widget::list_widget_item::ListWidgetItem;
use crate::widgets::with_scroll::with_scroll::WithScroll;

impl ListWidgetItem for Rc<String> {
    fn get_column_name(idx: usize) -> &'static str {
        match idx {
            0 => "name",
            _ => ""
        }
    }

    fn get_min_column_width(idx: usize) -> u16 {
        match idx {
            0 => 8,
            _ => 0
        }
    }

    fn len_columns() -> usize {
        1
    }

    fn get(&self, idx: usize) -> Option<Rc<String>> {
        match idx {
            0 => Some(self.clone()),
            _ => None
        }
    }
}

pub type ListWithScroll = WithScroll<ListWidget<Rc<String>>>;

pub struct WithScrollTestbed {
    pub editor_view: ListWithScroll,
    pub size: XY,
    pub config: ConfigRef,
    pub clipboard: ClipboardRef,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
    pub mock_navcomp_pilot: MockNavCompProviderPilot,
}


impl WithScrollTestbed {
    pub fn next_frame(&mut self) {
        let (mut output, rcvr) = MockOutput::new(self.size, false, self.theme.clone());

        self.editor_view.prelayout();
        self.editor_view.layout(output.size(), Rect::from_zero(output.size()));
        self.editor_view.render(&self.theme, true, &mut output);

        output.end_frame().unwrap();

        let frame = rcvr.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn interpreter(&self) -> Option<EditorInterpreter<'_>> {
        self.frame_op().map(|frame| {
            EditorInterpreter::new(frame, frame.metadata.first().unwrap())
        }).flatten()
    }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame|
            screenshot(&frame.buffer)
        ).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        recursive_treat_views(&mut self.editor_view, input);
        self.next_frame();
    }
}
