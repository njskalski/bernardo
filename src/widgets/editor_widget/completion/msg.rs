use crate::w7e::navcomp_provider::{Completion, CompletionAction};
use crate::widget::any_msg::AnyMsg;

#[derive(Clone, Debug)]
pub enum CompletionWidgetMsg {
    Close,
    Selected(CompletionAction),
}

impl AnyMsg for CompletionWidgetMsg {}
