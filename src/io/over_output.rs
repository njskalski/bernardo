use std::fmt::{Debug, Formatter};

use log::debug;
use log::warn;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::output::{Metadata, Output};
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::unpack_or;

// Over output is an output that is bigger than original,
// physical or in-memory display. All write operations targeting lines/columns beyond it's borders
// are to be silently disregarded.

pub struct OverOutput<'a> {
    output: &'a mut dyn Output,

    faked_size: XY,
    local_to_parent: XY,
}

impl<'a> OverOutput<'a> {
    pub fn new(
        output: &'a mut dyn Output,
        faked_size: XY,
        local_to_parent: XY,
    ) -> Self {
        if faked_size + local_to_parent < output.size() {
            warn!("seemingly unnecessary OverOutput, which fits entirely within parent output: faked_size: {}, offset: {}, source output: {}", faked_size, local_to_parent, output.size());
        }

        let res = OverOutput {
            output,
            faked_size,
            local_to_parent,
        };

        debug_assert!(res.validate_has_visible_rect());

        res
    }

    fn validate_has_visible_rect(&self) -> bool {
        self.output.visible_rect().minus_shift(self.local_to_parent).map(|new_visible_rect| new_visible_rect.capped_at(self.size())).flatten().is_some()
    }
}

impl SizedXY for OverOutput<'_> {
    fn size(&self) -> XY {
        self.faked_size
    }
}

impl Output for OverOutput<'_> {
    /*
    Again, remember, pos is in "widget space", not in space where "size constraint" was created.
     */
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        if text.width() > u16::MAX as usize {
            warn!("got text width that would overflow u16::MAX, not drawing.");
            debug!("early exit 0");
            return;
        }

        let local_visible_rect = self.output.visible_rect() + self.local_to_parent;

        if pos.y < local_visible_rect.pos.y {
            debug!("early exit 1");
            return;
        }
        // no analogue exit on x, as something starting left from frame might still overlap with it.

        if pos.y >= local_visible_rect.lower_right().y {
            debug!("drawing beyond output, early exit. pos: {} lvr: {}", pos, local_visible_rect);
            return;
        }

        let mut x_offset: i32 = 0;
        for grapheme in text.graphemes(true).into_iter() {
            let x = pos.x as i32 + x_offset - local_visible_rect.upper_left().x as i32;
            if x < 0 {
                continue;
            }
            if x > u16::MAX as i32 {
                warn!("got grapheme x position that would overflow u16::MAX, not drawing.");
                continue;
            }
            let x = x as u16;

            // if character would be drawn beyond output, drop it.
            if x + grapheme.width() as u16 >= local_visible_rect.lower_right().x {
                debug!("early exit 3");
                break;
            }

            let y = pos.y - local_visible_rect.upper_left().y; // >= 0, tested above and < u16::MAX since no addition.
            let local_pos = XY::new(x as u16, y);

            self.output.print_at(local_pos, style, grapheme);
            x_offset += grapheme.width() as i32; //TODO
        }
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        self.output.clear()
    }

    // TODO more tests
    fn visible_rect(&self) -> Rect {
        debug_assert!(self.validate_has_visible_rect());

        let parent_vis_rect = self.output.visible_rect();

        let my_rect = parent_vis_rect.minus_shift(self.local_to_parent).unwrap();
        let my_rect = my_rect.capped_at(self.size()).unwrap();

        debug_assert!(my_rect.lower_right() <= self.size());
        debug_assert!(my_rect.shifted(self.local_to_parent).lower_right() <= parent_vis_rect.lower_right());
        debug_assert!(parent_vis_rect.contains_rect(my_rect.shifted(self.local_to_parent)));

        my_rect
    }

    #[cfg(test)]
    fn emit_metadata(&mut self, mut meta: Metadata) {
        let upper_left = self.visible_rect().upper_left();

        if let Some(intersect_rect) = meta.rect.intersect(self.visible_rect()) {
            // this will give us intersection size
            meta.rect = intersect_rect;
            // but we also need to take account for the offset
            meta.rect.pos += upper_left;

            self.output.emit_metadata(meta);
        } else {
            debug!("discarding metadata, because i is no intersection: meta.typename {} meta.rect {}, visible_rect {}", meta.typename, meta.rect, self.visible_rect());
        }
    }
}

impl<'a> Debug for OverOutput<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "( OverOutput size {} offset {:?} over {:?} )", self.faked_size, self.local_to_parent, self.output)
    }
}