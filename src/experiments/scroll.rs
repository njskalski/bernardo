use stderrlog::new;

use crate::{Output, SizeConstraint, Theme, Widget};
use crate::experiments::scroll::ScrollDirection::Vertical;
use crate::io::over_output::OverOutput;
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, ZERO};

#[derive(PartialEq, Eq)]
pub enum ScrollDirection {
    Horizontal,
    Vertical,
    Both,
}

pub struct Scroll {
    offset : XY,
    direction : ScrollDirection
}

impl Scroll {
    pub fn new(direction: ScrollDirection) -> Self {
        Scroll {
            offset: ZERO,
            direction,
        }
    }

    pub fn render_within<W: Widget>(&self, output: &mut dyn Output, widget: &W, theme: &Theme, focused: bool) {
        let new_sc = match self.direction {
            ScrollDirection::Horizontal => SizeConstraint::new(
                None,
                Some(output.size_constraint().hint().size.y),
                Rect::new(self.offset, output.size_constraint().hint().size),
            ),
            ScrollDirection::Vertical => SizeConstraint::new(
                Some(output.size_constraint().hint().size.x),
                None,
                Rect::new(self.offset, output.size_constraint().hint().size),
            ),
            ScrollDirection::Both => SizeConstraint::new(
                None,
                None,
                Rect::new(self.offset, output.size_constraint().hint().size),
            ),
        };

        let mut over_output = OverOutput::new(output, new_sc);
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