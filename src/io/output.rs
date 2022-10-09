use std::borrow::Cow;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::io::buffer_output::BufferOutput;
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::widget::WID;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub id: WID,
    //could be static str, but I want serialize
    pub typename: String,
    pub rect: Rect,
    pub focused: bool,
}

pub trait Output: Debug {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str);
    fn clear(&mut self) -> Result<(), std::io::Error>;
    fn size_constraint(&self) -> SizeConstraint;

    /*
    Returns final screen position of given local pos, assuming that there is only one "FinalOutput".
    It will work with points up to .size() (so including first *not drawn* line and column) for ease
    of operation.
     */
    #[cfg(test)]
    fn get_final_position(&self, local_pos: XY) -> Option<XY>;

    #[cfg(test)]
    fn emit_metadata(&mut self, meta: Metadata);
}

pub trait FinalOutput: Output {
    fn end_frame(&mut self) -> Result<(), std::io::Error>;
    fn get_front_buffer(&self) -> &BufferOutput;
}
