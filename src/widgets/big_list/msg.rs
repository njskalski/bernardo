use crate::primitives::scroll_enum::ScrollEnum;
use crate::widget::any_msg::AnyMsg;

#[derive(Clone, Debug)]
pub enum BigListWidgetMsg {
    Scroll(ScrollEnum),
}

impl AnyMsg for BigListWidgetMsg {}

