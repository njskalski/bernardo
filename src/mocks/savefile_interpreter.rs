use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::treeview_interpreter::TreeViewInterpreter;
use crate::widgets::tree_view;

pub struct SaveFileInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    tree_view: TreeViewInterpreter<'a>,
}

impl<'a> SaveFileInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        let tree_view_meta: Vec<&Metadata> = output
            .get_meta_by_type(crate::mocks::savefile_interpreter::tree_view::tree_view::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(tree_view_meta.len() == 1);
        let tree_view = TreeViewInterpreter::new(tree_view_meta[0], output);

        Self {
            meta,
            output,
            tree_view,
        }
    }

    pub fn tree_view(&self) -> &TreeViewInterpreter<'a> {
        &self.tree_view
    }
}