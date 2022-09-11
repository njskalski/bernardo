use crate::AnyMsg;
use crate::w7e::navcomp_provider::Completion;

#[derive(Clone, Debug)]
pub enum CompletionWidgetMsg {
    Close,
    Selected(Completion),
}

impl AnyMsg for CompletionWidgetMsg {}
