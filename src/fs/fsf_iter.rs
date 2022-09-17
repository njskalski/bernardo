use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::Arc;

use log::{debug, error};
use streaming_iterator::StreamingIterator;

use crate::experiments::arc_vec_streaming_iter::ArcVecIter;
use crate::fs::path::SPath;

/*
Recursively iterates over all items under root, in DFS pattern, siblings sorted lexicographically.
 */
#[derive(Clone)]
pub struct RecursiveFsIter {
    // Invariant: first element - if exists - is non-empty
    stack: VecDeque<ArcVecIter<SPath>>,
}

impl RecursiveFsIter {
    pub fn new(root: SPath) -> Self {
        let stack: VecDeque<ArcVecIter<SPath>> = match root.blocking_list() {
            Ok(items) => {
                if items.is_empty() {
                    VecDeque::new()
                } else {
                    VecDeque::from([ArcVecIter::new(items)])
                }
            }
            Err(le) => {
                error!("swallowed list error : {:?}", le);
                VecDeque::new()
            }
        };

        RecursiveFsIter {
            stack,
        }
    }
}

impl StreamingIterator for RecursiveFsIter {
    type Item = SPath;

    fn advance(&mut self) {
        while let Some(front) = self.stack.front_mut() {
            front.advance();
            if front.get().is_some() {
                break;
            } else {
                self.stack.pop_front();
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        self.stack.front().map(|front| front.get()).flatten()
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