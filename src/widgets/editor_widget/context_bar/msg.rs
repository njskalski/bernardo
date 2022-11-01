use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::widget::any_msg::AnyMsg;

#[derive(Clone, Debug)]
pub enum ContextBarWidgetMsg {
    Close,
    Edit(CommonEditMsg),
}

impl AnyMsg for ContextBarWidgetMsg {}