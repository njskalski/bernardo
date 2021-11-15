use std::io::ErrorKind;

use log::debug;
use log::error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::output::Output;
use crate::io::style::{TextStyle, TextStyle_WhiteOnBlack};
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct SubOutput<'a> {
    output: Box<&'a mut dyn Output>,
    frame: Rect,
}

impl<'a> SubOutput<'a> {
    pub fn new(output: Box<&'a mut dyn Output>, frame: Rect) -> Self {
        // TODO add tests if frame is fully contained in Output and write errors to logs if its not.

        debug_assert!(frame.lower_right().x < output.size().x, "frame = {}, output.size() = {}", frame, output.size());
        debug_assert!(frame.lower_right().y < output.size().y, "frame = {}, output.size() = {}", frame, output.size());
        debug!("making suboutput {:?} from {:?}", frame, output.size());

        SubOutput { output, frame }
    }
}

impl SizedXY for SubOutput<'_> {
    fn size(&self) -> XY {
        self.frame.size
    }
}

impl Output for SubOutput<'_> {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        // TODO add grapheme cutting
        self.output.print_at(self.frame.pos + pos, style, text)
    }

    fn clear(&mut self) {
        // DO NOT clear the wider output, clear only your part.

        let style = TextStyle_WhiteOnBlack;

        for x in 0..self.frame.size.x {
            for y in 0..self.frame.size.y {
                self.output
                    .print_at(self.frame.pos + XY::new(x, y), style, " ")
            }
        }
    }
}
