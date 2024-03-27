use crate::io::output::Metadata;
use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::treeview_interpreter::TreeViewInterpreter;
use crate::widgets::context_menu;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::tree_view::tree_view;

pub struct ContextMenuInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    tree_view: TreeViewInterpreter<'a>,
    editbox: EditWidgetInterpreter<'a>,
}

// #[derive(Clone, Debug)]
// pub struct ContextMenuInterpreter {
//     pub label: String,
//     pub highlighted: bool,
// }

impl<'a> ContextMenuInterpreter<'a> {
    pub fn new(output: &'a MetaOutputFrame, meta: &'a Metadata) -> Self {
        debug_assert!(meta.typename == context_menu::widget::CONTEXT_MENU_WIDGET_NAME);

        let tree_view_meta: Vec<&Metadata> = output
            .get_meta_by_type(tree_view::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(tree_view_meta.len() == 1);
        let tree_view = TreeViewInterpreter::new(tree_view_meta[0], output);

        let editorbox_widget_meta: Vec<&Metadata> = output
            .get_meta_by_type(EditBoxWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(editorbox_widget_meta.len() == 1);
        let editbox = EditWidgetInterpreter::new(editorbox_widget_meta[0], output);

        ContextMenuInterpreter {
            meta,
            output,
            tree_view,
            editbox,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn tree_view(&self) -> &'a TreeViewInterpreter {
        &self.tree_view
    }

    pub fn editbox(&self) -> &'a EditWidgetInterpreter {
        &self.editbox
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
