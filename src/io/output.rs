

use crate::io::style::TextStyle;

use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub trait Output: SizedXY {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str);
    fn clear(&mut self);
    fn get_visible_rect(&self) -> Rect {
        Rect::new(XY::new(0, 0), self.size())
    }
}
