use std::fmt::{Debug, Formatter};
use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum Msg {

}


impl AnyMsg for Msg {}