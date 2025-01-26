use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::io::buffer_output::buffer_output::BufferOutput;

use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
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

pub trait Output: SizedXY + Debug {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str);
    fn clear(&mut self) -> Result<(), std::io::Error>;
    // fn size(&self) -> XY;
    fn visible_rect(&self) -> Rect;

    #[cfg(any(test, feature = "fuzztest"))]
    fn emit_metadata(&mut self, meta: Metadata);
}

pub trait FinalOutput: Output {
    fn end_frame(&mut self) -> Result<(), std::io::Error>;
    fn get_front_buffer(&self) -> &BufferOutput;
}
