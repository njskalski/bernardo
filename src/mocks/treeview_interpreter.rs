use unicode_segmentation::UnicodeSegmentation;

use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widgets::tree_view::tree_view;
use crate::widgets::tree_view::tree_view::TreeViewWidget;

pub struct TreeViewInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

#[derive(Clone, Debug)]
pub struct TreeViewInterpreterItem {
    pub label: String,
    pub depth: u16,
    pub leaf: bool,
    pub expanded: bool,
}

impl<'a> TreeViewInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == tree_view::TYPENAME);

        TreeViewInterpreter {
            meta,
            output,
        }
    }

    pub fn items(&self) -> Vec<TreeViewInterpreterItem> {
        let mut res: Vec<TreeViewInterpreterItem> = Vec::new();

        for line in self.output.buffer.lines_iter().with_rect(self.meta.rect) {
            if line.trim().is_empty() {
                continue;
            }

            let expanded = line.contains("▶");
            let is_dir = expanded || line.contains("▼");

            let line_no_sham = line.replace("▼", " ").replace("▶", " ");
            let mut first_non_blank: u16 = 0;
            for c in line_no_sham.graphemes(true) {
                if c == " " {
                    first_non_blank += 1;
                } else {
                    break;
                }
            }

            res.push(TreeViewInterpreterItem {
                label: line_no_sham.trim().to_string(),
                depth: (first_non_blank - 1) / 2,
                leaf: !is_dir,
                expanded,
            })
        }

        res
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }
}