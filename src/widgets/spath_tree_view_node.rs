use log::error;
use crate::new_fs::path::SPath;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

impl TreeViewNode<SPath> for SPath {
    fn id(&self) -> &SPath {
        self
    }

    fn label(&self) -> String {
        match self.last_name() {
            None => {
                error!("no last_name in SPath used in TreeViewNode");
                "<error>".to_string()
            }
            Some(item) => item.to_string(),
        }
    }

    fn is_leaf(&self) -> bool {
        todo!()
    }

    fn num_child(&self) -> (bool, usize) {
        todo!()
    }

    fn get_child(&self, idx: usize) -> Option<Self> {
        todo!()
    }

    fn is_complete(&self) -> bool {
        todo!()
    }
}