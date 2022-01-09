use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;

#[derive(Clone, Debug)]
pub enum MainViewMsg {
    FocusUpdateMsg(FocusUpdate)
}

impl AnyMsg for MainViewMsg {}