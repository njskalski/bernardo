use std::fmt::{Debug, Formatter};

use log::{debug, error};
use unicode_width::UnicodeWidthStr;

use crate::io::output::{Metadata, Output};
use crate::io::style::{TEXT_STYLE_WHITE_ON_BLACK, TextStyle};
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct SubOutput<'a> {
    output: &'a mut dyn Output,
    frame_in_parent_space: Rect,
}

impl<'a> SubOutput<'a> {
    pub fn new(output: &'a mut dyn Output, frame: Rect) -> Self {
        debug_assert!(frame.lower_right() <= output.size());
        debug_assert!(output.visible_rect().intersect(frame).is_some(), "no intersection between output.visible_rect() {} and frame of sub-output {}", output.visible_rect(), frame);

        SubOutput { output, frame_in_parent_space: frame }
    }
}

impl SizedXY for SubOutput<'_> {
    fn size(&self) -> XY {
        self.frame_in_parent_space.size
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

        let visible_rect = self.visible_rect();
        
        // if cfg!(debug_assertions) {
        //     // this <= is not an error, grapheme END can meet with frame END.
        //     debug_assert!(end_pos.x <= visible_rect.lower_right().x,
        //                   "drawing outside (to the right) the sub-output: ({} to {}) of {}",
        //                   pos, end_pos, self.frame_in_parent_space.size);
        //     debug_assert!(end_pos.y < self.frame_in_parent_space.size.y,
        //                   "drawing outside (below) the sub-output: ({} to {}) of {}",
        //                   pos, end_pos, self.frame_in_parent_space.size);
        // } else {
        //     if !(end_pos.x <= self.frame_in_parent_space.size.x && end_pos.y < self.frame_in_parent_space.size.y) {
        //         error!("drawing outside the sub-output: ({} to {}) of {}",
        //             pos, end_pos, self.frame_in_parent_space.size);
        //     }
        // }

        // TODO add grapheme cutting
        self.output.print_at(self.frame_in_parent_space.pos + pos, style, text)
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        // DO NOT clear the wider output, clear only your part.

        let style = TEXT_STYLE_WHITE_ON_BLACK;

        for x in 0..self.frame_in_parent_space.size.x {
            for y in 0..self.frame_in_parent_space.size.y {
                self.output
                    .print_at(self.frame_in_parent_space.pos + XY::new(x, y), style, " ")
            }
        }
        Ok(())
    }

    fn visible_rect(&self) -> Rect {
        // let parent_name = format!("{:?}", self.output);
        let parent_visible_rect = self.output.visible_rect();

        let my_visible_rect_in_parent_space = parent_visible_rect.intersect(self.frame_in_parent_space).unwrap(); // TODO unwrap

        let mut my_visible_space_in_my_space = my_visible_rect_in_parent_space.minus_shift(self.frame_in_parent_space.pos).unwrap();
        // my_visible_space_in_my_space.pos -= self.frame_in_parent_space.pos;

        let res = my_visible_space_in_my_space;

        debug_assert!(res.lower_right() <= self.size(), "res = {}, self.size() = {}", res, self.size());

        res
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
        meta.rect.pos = meta.rect.pos + self.frame_in_parent_space.pos;
        if meta.rect.lower_right() <= self.frame_in_parent_space.lower_right() {
            self.output.emit_metadata(meta)
        } else {
            debug!("suppressing metadata: {:?} - out of view", meta)
        }
    }
}

impl Debug for SubOutput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<SubOutput rect : {:?} of {:?} >", self.frame_in_parent_space, self.output)
    }
}