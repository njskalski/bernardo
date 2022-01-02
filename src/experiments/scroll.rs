use crate::{Output, Theme, Widget};
use crate::experiments::scroll::ScrollDirection::Vertical;
use crate::io::over_output::OverOutput;
use crate::primitives::xy::{XY, ZERO};

#[derive(PartialEq, Eq)]
pub enum ScrollDirection {
    Horizontal,
    Vertical,
    Both
}

pub struct Scroll {
    max_size : XY,
    offset : XY,
    direction : ScrollDirection
}

impl Scroll {
    pub fn new(max_size: XY) -> Self {
        Scroll {
            max_size,
            offset: ZERO,
            direction: ScrollDirection::Vertical,
        }
    }

    pub fn set_max_size(&mut self, new_max_size: XY) {
        self.max_size = new_max_size;
        // self.offset = self.offset.cut(new_max_size); //TODO it should be -1 -1 I guess
    }

    pub fn render_within<W: Widget>(&self, output: &mut dyn Output, widget: &W, theme: &Theme, focused: bool) {
        let mut over_output = OverOutput::new(output, self.offset, self.max_size);
        widget.render(theme, focused, &mut over_output)
    }

    pub fn follow_anchor(&mut self, output_size: XY, anchor: XY) {
        let adjust_x = self.direction == ScrollDirection::Horizontal || self.direction == ScrollDirection::Both;
        let adjust_y = self.direction == ScrollDirection::Vertical || self.direction == ScrollDirection::Both;

        if adjust_x {
            if anchor.x < self.offset.x {
                self.offset.x = anchor.x;
            }

            if anchor.x >= (self.offset.x + output_size.x) {
                let diff = 1 + anchor.x - (self.offset.x + output_size.x);
                self.offset.x += diff;
            }
        }

        if adjust_y {
            if anchor.y < self.offset.y {
                self.offset.y = anchor.y;
            }

            if anchor.y >= (self.offset.y + output_size.y) {
                let diff = 1 + anchor.y - (self.offset.y + output_size.y);
                self.offset.y += diff;
            }
        }
    }
}