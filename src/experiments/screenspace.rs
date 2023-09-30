use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;

/*
Screenspace represents a combined information about output size:
- each output starts in (0,0) and is of output_size (exclusive)
- visible rect is a non-deformed, strictly contained piece of screen that will be visible to the
    user. The reason this information is passed is:
    1) to facilitate meaningful PageUp/PageDown operations
    2) to optimise rendering
 */
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Screenspace {
    output_size: XY,
    visible_rect: Rect,
}

impl Screenspace {
    pub fn new(output_size: XY, visible_rect: Rect) -> Self {
        debug_assert!(output_size.has_non_zero_area());
        debug_assert!(visible_rect.lower_right() <= output_size, "visible_rect = {}, output_size = {}", visible_rect, output_size);
        debug_assert!(visible_rect.size.has_non_zero_area());

        Self {
            output_size,
            visible_rect,
        }
    }

    pub fn full_output(output_size: XY) -> Self {
        debug_assert!(output_size.has_non_zero_area());

        Self {
            output_size,
            visible_rect: Rect::new(XY::ZERO, output_size),
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