use crate::AnyMsg;
use crate::w7e::navcomp_provider::{Completion, CompletionAction};

#[derive(Clone, Debug)]
pub enum CompletionWidgetMsg {
    Close,
    Selected(CompletionAction),
}

impl AnyMsg for CompletionWidgetMsg {}
