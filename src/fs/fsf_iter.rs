use log::error;

use crate::fs::path::SPath;

/*
Recursively iterates over all items under root, in DFS pattern, siblings sorted lexicographically
 */
pub struct RecursiveFsIter {
    stack: Vec<Box<dyn Iterator<Item = SPath>>>,
}

impl RecursiveFsIter {
    pub fn new(root: SPath) -> Self {
        let first_iter: Box<dyn Iterator<Item = SPath>> = match root.blocking_list() {
            Ok(mut items) => {
                items.sort();
                Box::new(items.into_iter())
            }
            Err(le) => {
                error!("swallowed list error : {:?}", le);
                Box::new(std::iter::empty())
            }
        };

        RecursiveFsIter {
            stack: Vec::from([first_iter]),
        }
    }
}

impl Iterator for RecursiveFsIter {
    type Item = SPath;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(iter) = self.stack.last_mut() {
            let Some(item) = iter.next() else {
                // The iter is empty; remove it
                self.stack.pop();
                continue;
            };

            if item.is_hidden() {
                continue;
            }

            if item.is_dir() {
                match item.blocking_list() {
                    Ok(mut children) => {
                        children.sort();
                        self.stack.push(Box::new(children.into_iter()));
                    }
                    Err(le) => error!("swallowed list error 2 : {:?}", le),
                };
            }

            return Some(item);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::filesystem_front::FilesystemFront;
    use crate::fs::fsf_iter::RecursiveFsIter;
    use crate::fs::mock_fs::MockFS;
    use crate::spath;

    #[test]
    fn test_all_iter() {
        let m = MockFS::new("/tmp")
            .with_file("folder1/folder2/file1.txt", "some text")
            .with_file("folder1/folder3/moulder.txt", "truth is out there")
            .to_fsf();

        let mut iter = RecursiveFsIter::new(m.root());

        assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2", "file1.txt").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_hidden_dir_is_ignored() {
        let m = MockFS::new("/tmp")
            .with_file("folder1/folder2/file1.txt", "some text")
            .with_file("folder1/.git/moulder.txt", "truth is out there")
            .to_fsf();

        let mut iter = RecursiveFsIter::new(m.root());

        assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2", "file1.txt").unwrap()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic]
    fn test_root_gitignore_is_respected() {
        let m = MockFS::new("/tmp")
            .with_file(".gitignore", "file1.txt")
            .with_file("folder1/folder2/file1.txt", "not matched")
            .with_file("folder1/folder3/moulder.txt", "truth is out there")
            .to_fsf();

        let mut iter = RecursiveFsIter::new(m.root());

        assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic]
    fn test_nested_gitignore_is_respected() {
        let m = MockFS::new("/tmp")
            .with_file("folder1/folder2/.gitignore", "file1.txt")
            .with_file("folder1/folder2/file1.txt", "not matched")
            .with_file("folder1/folder3/moulder.txt", "truth is out there")
            .with_file("folder1/folder3/file1.txt", "matched")
            .to_fsf();

        let mut iter = RecursiveFsIter::new(m.root());

        assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "file1.txt").unwrap()));
        assert_eq!(iter.next(), None);
    }
}
