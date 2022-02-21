#[derive(Debug)]
pub struct FilesystemTreeNode2 {
    path: PathBuf,

    filesystem_view: Rc<FilesystemView>,

    finished: bool,
}