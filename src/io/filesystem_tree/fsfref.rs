use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use crate::io::filesystem_tree::file_front::FileFront;
use crate::io::filesystem_tree::filesystem_front::FilesystemFront;

#[derive(Clone, Debug)]
pub struct FsfRef(pub Rc<Box<dyn FilesystemFront>>);

impl FsfRef {
    pub fn get_root(&self) -> FileFront {
        FileFront::new(
            self.clone(),
            self.0.get_root_path().clone(),
        )
    }

    pub fn get_children(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=FileFront> + '_>) {
        let (done, it) = self.0.get_children_paths(path);
        let new_it = it.map(move |p| FileFront::new(self.clone(), p.clone()));

        (done, Box::new(new_it))
    }

    pub fn get_item(&self, path: &Path) -> Option<FileFront> {
        self.get_path(path).map(|p| {
            FileFront::new(self.clone(), p)
        })
    }
}

impl AsRef<dyn FilesystemFront> for FsfRef {
    fn as_ref(&self) -> &(dyn FilesystemFront + 'static) {
        self.0.as_ref().as_ref()
    }
}

impl Deref for FsfRef {
    type Target = dyn FilesystemFront;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().as_ref()
    }
}

impl PartialEq for FsfRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for FsfRef {}

impl Hash for FsfRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state)
    }
}