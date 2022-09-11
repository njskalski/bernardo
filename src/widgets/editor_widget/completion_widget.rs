/*
I guess I should reuse FuzzySearch Widget, this is a placeholder now.
 */

use std::future::Future;

use futures::FutureExt;

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
use crate::primitives::xy::XY;
use crate::w7e::navcomp_provider::Completion;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::fuzzy_search::fuzzy_search::FuzzySearchWidget;

pub type CompletionsFuture = Box<dyn Future<Output=Vec<Completion>>>;


#[derive(Clone, Debug)]
pub enum CompletionWidgetMsg {
    Close,

}

impl AnyMsg for CompletionWidgetMsg {}

pub struct CompletionWidget {
    wid: WID,
    completions_future: CompletionsFuture,
    completions: Option<Vec<Completion>>,
    fuzzy: FuzzySearchWidget,
}

impl CompletionWidget {
    pub fn new(completions_future: CompletionsFuture) -> Self {
        CompletionWidget {
            wid: get_new_widget_id(),
            fuzzy: FuzzySearchWidget::new(|_| {
                CompletionWidgetMsg::Close.someboxed()
            }),
            completions_future,
            completions: None,
        }
    }

    // fn completions(&mut self) -> Option<&Vec<Completion>> {
    //     if self.completions.is_some() {
    //         self.completions.as_ref()
    //     } else {
    //         if let Some(item) = self.completions_future.now_or_never() {
    //             self.completions = Some(item);
    //         }
    //         self.completions.as_ref()
    //     }
    // }
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