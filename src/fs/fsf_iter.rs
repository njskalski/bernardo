use std::collections::VecDeque;

use log::{debug, error};

use crate::fs::path::SPath;

/*
Recursively iterates over all items under root, in DFS pattern, siblings sorted lexicographically
 */
pub struct RecursiveFsIter {
    stack: VecDeque<Box<dyn Iterator<Item=SPath>>>,
}

impl RecursiveFsIter {
    pub fn new(root: SPath) -> Self {
        let first_iter: Box<dyn Iterator<Item=SPath>> = match root.blocking_list() {
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
            stack: VecDeque::from([first_iter])
        }
    }
}

impl Iterator for RecursiveFsIter {
    type Item = SPath;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }


        while let Some(iter) = self.stack.front_mut() {
            if let Some(item) = iter.next() {
                if item.is_dir() {
                    match item.blocking_list() {
                        Ok(mut children) => {
                            children.sort();
                            self.stack.push_front(Box::new(children.into_iter()));
                        }
                        Err(le) => {
                            error!("swallowed list error 2 : {:?}", le);
                        }
                    };
                }

                return Some(item);
            } else {
                self.stack.pop_front();
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{FilesystemFront, spath};
    use crate::fs::fsf_iter::RecursiveFsIter;
    use crate::fs::mock_fs::MockFS;

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
}