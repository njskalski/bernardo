/*
This describes a continous horizontal piece of buffer output, containing "text" and perhaps having a
consistent style.
 */
use crate::io::style::TextStyle;
use crate::primitives::xy::XY;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HorizontalIterItem {
    pub absolute_pos: XY,
    // Set iff style was consistent over entire item
    pub text_style: Option<TextStyle>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsistentHorizontalItem {
    pub absolute_pos: XY,
    pub text_style: TextStyle,
    pub text: String,
}
