use std::fmt::{Debug, Formatter};

use log::debug;
use log::warn;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::output::{Metadata, Output};
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::unpack_or;

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
    /*
    Again, remember, pos is in "widget space", not in space where "size constraint" was created.
     */
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        let visible_rect = match self.size_constraint.visible_hint() {
            None => {
                debug!("skipping drawing - no visible part");
                return;
            }
            Some(v) => v,
        };

        if text.width() > u16::MAX as usize {
            warn!("got text width that would overflow u16::MAX, not drawing.");
            // debug!("early exit 0");
            return;
        }

        if pos.y < visible_rect.pos.y {
            // debug!("early exit 1");
            return;
        }
        // no analogue exit on x, as something starting left from frame might still overlap with it.

        if pos.y >= visible_rect.lower_right().y {
            // debug!("early exit 2");
            debug!("drawing beyond output, early exit. pos: {} sc: {}", pos, self.size_constraint());
            return;
        }

        let mut x_offset: i32 = 0;
        for grapheme in text.graphemes(true).into_iter() {
            let x = pos.x as i32 + x_offset - visible_rect.upper_left().x as i32;
            if x < 0 {
                continue;
            }
            if x > u16::MAX as i32 {
                warn!("got grapheme x position that would overflow u16::MAX, not drawing.");
                continue;
            }
            let x = x as u16;

            if self.output.size_constraint().x().map(|max_x| max_x <= x).unwrap_or(true) {
                // debug!("early exit 3");
                break;
            }

            let y = pos.y - visible_rect.upper_left().y; // >= 0, tested above and < u16::MAX since no addition.
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

    #[cfg(test)]
    fn emit_metadata(&mut self, mut meta: Metadata) {
        let visible_rect = unpack_or!(self.size_constraint.visible_hint(), (), "not emiting, no visible part");
        let upper_left = visible_rect.upper_left();

        if meta.rect.pos.x >= upper_left.x && meta.rect.pos.y >= upper_left.y {
            meta.rect.pos = meta.rect.pos - upper_left;
            if meta.rect.lower_right() <= visible_rect.lower_right() {
                self.output.emit_metadata(meta);
            } else {
                debug!("discarding metadata, because it's below the view: {:?} vs {:?}", meta, visible_rect);
            }
        } else {
            debug!("discarding metadata, because it's above the view: {:?} vs {:?}", meta, visible_rect);
        }
    }
}

impl<'a> Debug for OverOutput<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "( OverOutput sc {:?} over {:?} )", self.size_constraint, self.output)
    }
}