use crate::io::style::TextStyle;
use crate::primitives::xy::XY;
use crate::SizeConstraint;

pub trait Output {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str);
    fn clear(&mut self);
    fn size_constraint(&self) -> SizeConstraint;
}
