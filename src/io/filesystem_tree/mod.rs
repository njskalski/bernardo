use std::fmt::{Display, Formatter};

pub mod filesystem_front;
pub mod local_filesystem_front;
pub mod file_front;
pub mod fsfref;
mod internal_state;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LoadingState {
    Complete,
    InProgress,
    NotStarted,
    Error,
}

impl Display for LoadingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}