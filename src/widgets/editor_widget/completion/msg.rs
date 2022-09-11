use crate::AnyMsg;

#[derive(Clone, Debug)]
pub enum CompletionWidgetMsg {
    Close,

}

impl AnyMsg for CompletionWidgetMsg {}
