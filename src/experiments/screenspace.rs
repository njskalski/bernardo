use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Screenspace {
    output_size: XY,
    visible_rect: Rect,
}

impl Screenspace {
    pub fn new(output_size: XY, visible_rect: Rect) -> Self {
        debug_assert!(output_size.has_non_zero_area());
        // TODO this is not always true now
        // debug_assert!(visible_rect.lower_right() <= output_size);
        debug_assert!(visible_rect.size.has_non_zero_area());

        Self {
            output_size,
            visible_rect,
        }
    }
    pub fn output_size(&self) -> XY {
        self.output_size
    }

    pub fn visible_rect(&self) -> Rect {
        self.visible_rect
    }

    pub fn page_height(&self) -> u16 {
        self.visible_rect.size.y
    }
}