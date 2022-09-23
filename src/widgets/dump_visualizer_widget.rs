use std::cmp::min;
use std::mem;

use crate::config::theme::Theme;
use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

pub struct DumpVisualizerWidget {
    wid: WID,
    dump_op: Option<BufferOutput>,
}

impl DumpVisualizerWidget {
    pub fn new() -> Self {
        Self {
            wid: get_new_widget_id(),
            dump_op: None,
        }
    }

    pub fn with_dump(self, dump: BufferOutput) -> Self {
        Self {
            dump_op: Some(dump),
            ..self
        }
    }

    pub fn set_dump(&mut self, mut dump_op: Option<BufferOutput>) -> Option<BufferOutput> {
        mem::swap(&mut self.dump_op, &mut dump_op);
        dump_op
    }
}

impl Widget for DumpVisualizerWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "DumpVisualizerWidget"
    }

    fn min_size(&self) -> XY {
        XY::new(10, 10)
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        sc.visible_hint().size
    }

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, _theme: &Theme, _focused: bool, output: &mut dyn Output) {
        if let Some(dump) = self.dump_op.as_ref() {
            let max_x = min(dump.size().x, output.size_constraint().visible_hint().size.x);
            let max_y = min(dump.size().y, output.size_constraint().visible_hint().size.y);

            for x in 0..max_x {
                for y in 0..max_y {
                    let xy = XY::new(x, y);
                    let cell = &dump[xy];
                    match cell {
                        Cell::Continuation => {}
                        Cell::Begin { style, grapheme } => {
                            output.print_at(
                                XY::new(x, y),
                                *style,
                                grapheme,
                            )
                        }
                    }
                }
            }
        }
    }
}