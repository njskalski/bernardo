use log::debug;
use log::warn;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::primitives::xy::XY;
use crate::SizeConstraint;

// Over output (maybe I'll rename it as super output) is an output that is bigger than original,
// physical or in-memory display. All write operations targeting lines/columns beyond it's borders
// are to be silently disregarded.

pub struct OverOutput<'a> {
    output: &'a mut dyn Output,
    size_constraint: SizeConstraint,
}

impl<'a> OverOutput<'a> {
    pub fn new(
        output: &'a mut dyn Output,
        size_constraint: SizeConstraint,
    ) -> Self {
        // debug!("making overoutput {:?}", size_constraint);
        OverOutput {
            output,
            size_constraint,
        }
    }
}

impl Output for OverOutput<'_> {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        // TODO: I have no clue why this was failing
        if !self.size_constraint.bigger_equal_than(
            self.size_constraint.visible_hint().lower_right()
        ) {
            warn!("hint (visible part) beyond output space. Most likely layouting error. Hint: {}, output: {}",
                self.size_constraint.visible_hint(),
                self.size_constraint,
            );
        }

        if text.width() > u16::MAX as usize {
            warn!("got text width that would overflow u16::MAX, not drawing.");
            return;
        }

        if pos.y < self.size_constraint.visible_hint().upper_left().y {
            debug!("early exit 1");
            return;
        }
        // no analogue exit on x, as something starting left from frame might still overlap with it.

        if !self.size_constraint().bigger_equal_than(pos) {
            debug!("drawing beyond output, early exit. pos: {} sc: {}", pos, self.size_constraint());
            // return;
        }

        let mut x_offset: i32 = 0;
        for grapheme in text.graphemes(true).into_iter() {
            let x = pos.x as i32 + x_offset - self.size_constraint.visible_hint().upper_left().x as i32;
            if x < 0 {
                continue;
            }
            if x > u16::MAX as i32 {
                warn!("got grapheme x position that would overflow u16::MAX, not drawing.");
                continue;
            }
            let x = x as u16;

            if self.output.size_constraint().x().map(|max_x| max_x <= x).unwrap_or(true) {
                break;
            }

            let y = pos.y - self.size_constraint.visible_hint().upper_left().y; // >= 0, tested above and < u16::MAX since no addition.
            let local_pos = XY::new(x as u16, y);

            self.output.print_at(local_pos, style, grapheme);
            x_offset += grapheme.width() as i32; //TODO
        }
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        self.output.clear()
    }

    fn size_constraint(&self) -> SizeConstraint {
        self.size_constraint
    }
}
