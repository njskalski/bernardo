use std::cmp::min;
use std::mem;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::unpack_unit;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, Widget, WID};

pub struct DumpVisualizerWidget {
    wid: WID,
    dump_op: Option<BufferOutput>,

    last_size: Option<Screenspace>,
}

impl DumpVisualizerWidget {
    pub const TYPENAME: &'static str = "DumpVisualizerWidget";

    pub fn new() -> Self {
        Self {
            wid: get_new_widget_id(),
            dump_op: None,
            last_size: None,
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
        Self::TYPENAME
    }
    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }

    fn full_size(&self) -> XY {
        self.dump_op.as_ref().map(|oo| oo.size()).unwrap_or(XY::new(10, 10))
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.last_size = Some(screenspace)
    }

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, _theme: &Theme, _focused: bool, output: &mut dyn Output) {
        let size = unpack_unit!(self.last_size, "render before layout",);

        if let Some(dump) = self.dump_op.as_ref() {
            let max_x = min(dump.size().x, size.output_size().x);
            let max_y = min(dump.size().y, size.output_size().y);

            for x in 0..max_x {
                for y in 0..max_y {
                    let xy = XY::new(x, y);
                    let cell = &dump[xy];
                    match cell {
                        Cell::Continuation => {}
                        Cell::Begin { style, grapheme } => output.print_at(XY::new(x, y), *style, grapheme),
                    }
                }
            }
        }
    }
}
