use std::rc::Rc;

use crate::config::theme::Theme;
use crate::experiments::screen_shot::screenshot;
use crate::gladius::paradigm::recursive_treat_views;
use crate::io::input_event::InputEvent;
use crate::io::output::FinalOutput;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::with_scroll_interpreter::WithScrollWidgetInterpreter;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
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
    pub widget: ListWithScroll,
    pub size: XY,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
}


impl WithScrollTestbed {
    pub fn new() -> Self {
        let list_widget = ListWidget::<Rc<String>>::new();

        Self {
            widget: ListWithScroll::new(ScrollDirection::Vertical, list_widget),
            size: XY::new(10, 20),
            theme: Default::default(),
            last_frame: None,
        }
    }


    pub fn next_frame(&mut self) {
        let (mut output, rcvr) = MockOutput::new(self.size, false, self.theme.clone());

        self.widget.prelayout();
        self.widget.layout(output.size(), Rect::from_zero(output.size()));
        self.widget.render(&self.theme, true, &mut output);

        output.end_frame().unwrap();

        let frame = rcvr.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn interpreter(&self) -> Option<WithScrollWidgetInterpreter<'_, ListWidget<Rc<String>>>> {
        self.frame_op().map(|frame| {
            let meta = frame.metadata.iter().find(|item| {
                item.typename == ListWithScroll::static_typename()
            }).unwrap();

            WithScrollWidgetInterpreter::new(frame, meta)
        })
    }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame|
            screenshot(&frame.buffer)
        ).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        recursive_treat_views(&mut self.widget, input);
        self.next_frame();
    }
}
