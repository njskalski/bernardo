use crate::io::buffer_output_iter::VerticalIterItem;
use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::primitives::xy::XY;

pub struct CompletionInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

impl<'a> CompletionInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        Self {
            meta,
            output,
        }
    }

    pub fn items(&self) -> impl Iterator<Item=VerticalIterItem> + '_ {
        // for d in self.output.buffer.lines_iter().with_rect(self.meta.rect) {
        //     debug!("items: [{}]", d);
        // }

        self.output.buffer.lines_iter().with_rect(self.meta.rect)
    }

    pub fn highlighted(&self, highlighted: bool) -> Option<(u16, String)> {
        // So here the issue is, that since completions are fuzzy filtered, no common text style
        // appears over an entire item (some letters are highlighted). Therefore a specialized code
        // is created.
        let first_column = self.meta.rect.pos.x;

        let mut idx: Option<u16> = None;

        for y in self.meta.rect.pos.y..self.meta.rect.lower_right().y {
            let pos = XY::new(first_column, y);
            let cell = &self.output.buffer[pos];
            if cell.style().unwrap().background == self.output.theme.highlighted(highlighted).background {
                idx = Some(y - self.meta.rect.pos.y);
                break;
            }
        }

        idx.map(|idx| {
            (idx, self.output.buffer.lines_iter().with_rect(self.meta.rect).skip(idx as usize).next().unwrap().text)
        })
    }
}