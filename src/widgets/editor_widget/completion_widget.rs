/*
I guess I should reuse FuzzySearch Widget, this is a placeholder now.
 */

use std::future::Future;

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
use crate::primitives::xy::XY;
use crate::w7e::navcomp_provider::Completion;
use crate::widget::widget::{get_new_widget_id, WID};

pub type CompletionsFuture = Box<dyn Future<Output=Vec<Completion>>>;

pub struct CompletionWidget {
    wid: WID,
    fuzzy: bool,

    completions: CompletionsFuture,
}

impl CompletionWidget {
    pub fn new(completions_future: CompletionsFuture) -> Self {
        CompletionWidget {
            wid: get_new_widget_id(),
            fuzzy: true,
            completions: completions_future,
        }
    }
}

impl Widget for CompletionWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "CompletionWidget"
    }

    fn min_size(&self) -> XY {
        (3, 10).into()
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        todo!()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }
}