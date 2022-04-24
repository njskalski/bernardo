/*
This is a piece of specialized code for TreeView of Rc<PathBuf>
 */

use std::path::PathBuf;
use std::rc::Rc;
use log::{debug, error, warn};
use crate::fs::file_front::FileFront;
use crate::FsfRef;
use crate::widgets::tree_view::tree_view::TreeViewWidget;

impl TreeViewWidget<PathBuf, FileFront> {
    pub fn set_path(&mut self, fsf: &FsfRef, path: &Rc<PathBuf>) -> bool {
        debug!("setting path to {:?}", path);

        let (mut dir, filename): (PathBuf, Option<&str>) = if fsf.is_dir(path) {
            (path.to_path_buf(), None)
        } else {
            (path.parent().map(|f| f.to_path_buf()).unwrap_or_else(|| {
                warn!("failed to extract parent of {:?}, defaulting to fsf.root", path);
                fsf.get_root_path().to_path_buf()
            }), path.file_name().map(|s| s.to_str()).unwrap_or_else(|| {
                warn!("filename at end of {:?} is not UTF-8", path);
                None
            }))
        };

        if !dir.starts_with(fsf.get_root_path().as_path()) {
            error!("attempted to set path to non-root location {:?}, defaulting to fsf.root", dir);
            dir = fsf.get_root_path().to_path_buf();
        }

        // now I will be stripping pieces of dir path and expanding each of them (bottom-up)
        self.expanded_mut().insert(dir.to_path_buf());

        let mut root_path = fsf.get_root_path().to_path_buf();
        self.expanded_mut().insert(root_path.clone());

        match dir.strip_prefix(&root_path) {
            Err(e) => {
                error!("supposed to set path to {:?}, but it's outside fs {:?}, because: {}", path, &root_path, e);
                return false;
            }
            Ok(remainder) => {
                for comp in remainder.components() {
                    root_path = root_path.join(comp);
                    debug!("expanding subtree {:?}", &root_path);
                    self.expanded_mut().insert(root_path.clone());
                }
            }
        }

        if !self.set_selected(&root_path) {
            error!("failed to select {:?}", root_path);
            return false;
        }

        true
    }
}