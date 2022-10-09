use crate::io::output::Metadata;
use crate::mocks::listview_interpreter::ListViewInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::treeview_interpreter::TreeViewInterpreter;
use crate::widgets::list_widget::list_widget;
use crate::widgets::tree_view::tree_view;

pub struct SaveFileInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    tree_view: TreeViewInterpreter<'a>,
    list_view: ListViewInterpreter<'a>,
}

impl<'a> SaveFileInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        let tree_view_meta: Vec<&Metadata> = output
            .get_meta_by_type(tree_view::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(tree_view_meta.len() == 1);
        let tree_view = TreeViewInterpreter::new(tree_view_meta[0], output);

        let list_view_meta: Vec<&Metadata> = output
            .get_meta_by_type(list_widget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(list_view_meta.len() == 1);
        let list_view = ListViewInterpreter::new(list_view_meta[0], output);

        Self {
            meta,
            output,
            tree_view,
            list_view,
        }
    }

    pub fn tree_view(&self) -> &TreeViewInterpreter<'a> {
        &self.tree_view
    }

    pub fn list_view(&self) -> &ListViewInterpreter<'a> { &self.list_view }
}