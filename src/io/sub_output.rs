use std::fmt::{Debug, Formatter};

use log::{debug, error};
use unicode_width::UnicodeWidthStr;

use crate::io::output::{Metadata, Output};
use crate::io::style::{TEXT_STYLE_WHITE_ON_BLACK, TextStyle};
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct SubOutput<'a> {
    output: &'a mut dyn Output,
    frame: Rect,
}

impl<'a> SubOutput<'a> {
    pub fn new(output: &'a mut dyn Output, frame: Rect) -> Self {
        debug_assert!(frame.lower_right() <= output.size());
        debug_assert!(output.visible_rect().intersect(&frame).is_some());

        SubOutput { output, frame }
    }
}

impl SizedXY for SubOutput<'_> {
    fn size(&self) -> XY {
        self.frame.size
    }
}

impl Output for SubOutput<'_> {
    /*
    Here pos is in "local space" of widget drawing to this SubOutput, which generally assumes it can
    draw from (0,0) to widget.size()
    and self.frame() is on "parent space".
    So we compare for "drawing beyond border" against *size* of the frame, not it's position.
     */
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        let end_pos = pos + (text.width() as u16, 0);

        if cfg!(debug_assertions) {
            debug_assert!(end_pos.x <= self.frame.size.x,
                          "drawing outside (to the right) the sub-output: ({} to {}) of {}",
                          pos, end_pos, self.frame.size);
            debug_assert!(end_pos.y < self.frame.size.y,
                          "drawing outside (below) the sub-output: ({} to {}) of {}",
                          pos, end_pos, self.frame.size);
        } else {
            if !(end_pos.x <= self.frame.size.x && end_pos.y < self.frame.size.y) {
                error!("drawing outside the sub-output: ({} to {}) of {}",
                    pos, end_pos, self.frame.size);
            }
        }

        // TODO add grapheme cutting
        self.output.print_at(self.frame.pos + pos, style, text)
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        // DO NOT clear the wider output, clear only your part.

        let style = TEXT_STYLE_WHITE_ON_BLACK;

        for x in 0..self.frame.size.x {
            for y in 0..self.frame.size.y {
                self.output
                    .print_at(self.frame.pos + XY::new(x, y), style, " ")
            }
        }
        Ok(())
    }

    fn visible_rect(&self) -> Rect {
        self.output.visible_rect().intersect(&self.frame).unwrap()
    }


    // #[cfg(test)]
    // fn get_final_position(&self, local_pos: XY) -> Option<XY> {
    //     let parent_pos = local_pos + self.frame.pos;
    //     if parent_pos <= self.frame.lower_right() {
    //         self.output.get_final_position(parent_pos)
    //     } else {
    //         None
    //     }
    // }

    #[cfg(test)]
    fn emit_metadata(&mut self, mut meta: Metadata) {
        meta.rect.pos = meta.rect.pos + self.frame.pos;
        if meta.rect.lower_right() <= self.frame.lower_right() {
            self.output.emit_metadata(meta)
        } else {
            debug!("suppressing metadata: {:?} - out of view", meta)
        }
    }
}

impl Debug for SubOutput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<SubOutput rect : {:?} of {:?} >", self.frame, self.output)
    }
}