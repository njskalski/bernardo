use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widgets::list_widget::list_widget;

pub struct ListViewInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

#[derive(Clone, Debug)]
pub struct ListViewInterpreterItem {
    pub label: String,
    pub highlighted: bool,
}

impl<'a> ListViewInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == list_widget::TYPENAME);

        ListViewInterpreter { meta, output }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    // pub fn items(&self) -> Vec<TreeViewInterpreterItem> {
    //     let mut res: Vec<TreeViewInterpreterItem> = Vec::new();
    //
    //     for (line_idx, line) in self.output.buffer.lines_iter().with_rect(self.meta.rect).enumerate()
    // {         if line.trim().is_empty() {
    //             continue;
    //         }
    //
    //         let expanded = line.contains("▶");
    //         let is_dir = expanded || line.contains("▼");
    //
    //         let line_no_sham = line.replace("▼", " ").replace("▶", " ");
    //         let mut first_non_blank: u16 = 0;
    //         for c in line_no_sham.graphemes(true) {
    //             if c == " " {
    //                 first_non_blank += 1;
    //             } else {
    //                 break;
    //             }
    //         }
    //
    //         let pos_first = self.meta.rect.pos + XY::new(first_non_blank, line_idx as u16);
    //         let highlighted = self.output.buffer[pos_first].style().unwrap().background ==
    // self.output.theme.highlighted(true).background;
    //
    //         res.push(TreeViewInterpreterItem {
    //             label: line_no_sham.trim().to_string(),
    //             depth: (first_non_blank - 1) / 2,
    //             leaf: !is_dir,
    //             expanded,
    //             highlighted,
    //         })
    //     }
    //
    //     res
    // }
}
