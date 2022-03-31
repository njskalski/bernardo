use std::ops::Deref;
use std::rc::Rc;
use crate::io::filesystem_tree::file_front::FileFront;
use crate::io::filesystem_tree::filesystem_front::FilesystemFront;

#[derive(Clone, Debug)]
pub struct FsfRef(pub Rc<Box<dyn FilesystemFront>>);

impl FsfRef {
    pub fn get_root(&self) -> FileFront {
        FileFront {
            fsf: self.clone(),
            path: self.0.get_root_path().clone(),
        }
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