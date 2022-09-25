use std::any::Any;
use std::fmt::Debug;

use crate::io::buffer_output::BufferOutput;
use crate::io::style::TextStyle;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;

pub trait Output: Debug {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str);
    fn clear(&mut self) -> Result<(), std::io::Error>;
    fn size_constraint(&self) -> SizeConstraint;
}

pub trait FinalOutput: Output {
    fn end_frame(&mut self) -> Result<(), std::io::Error>;
    fn get_front_buffer(&self) -> &BufferOutput;
}
