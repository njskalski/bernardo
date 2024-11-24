use std::fmt::Debug;

use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum GladiusMsg {
    Quit,
    Screenshot,
}


impl AnyMsg for GladiusMsg {}