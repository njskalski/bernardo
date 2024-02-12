use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::error;

use crate::fs::path::SPath;

fn parse_gitignore(ignore_path: SPath) -> Option<Gitignore> {
    let absolute_path = ignore_path.absolute_path();
    let contents = ignore_path.read_entire_file_to_string().ok()?;

    let mut builder = GitignoreBuilder::new(ignore_path.parent()?.absolute_path());
    for line in contents.lines() {
        // GitignoreBuilder::add() can't be used because it uses an File::open()
        // call on the filepath, but we use our own FS layer
        let _ = builder.add_line(Some(absolute_path.clone()), line);
    }
    builder.build().ok()
}

const GITIGNORE_FILE: &str = ".gitignore";

/*
Recursively iterates over all items under root, in DFS pattern, siblings sorted lexicographically
 */
pub struct RecursiveFsIter {
    ignore: Option<Gitignore>,
    stack: Vec<Box<dyn Iterator<Item = SPath>>>,
}

impl RecursiveFsIter {
    pub fn new(root: SPath) -> Self {
        let (first_iter, ignore): (Box<dyn Iterator<Item = SPath>>, _) = match root.blocking_list() {
            Ok(mut items) => {
                items.sort();
                let ignore = items
                    // It should be fine to compare by the basename since items only contains
                    // files from a single directory
                    .binary_search_by(|path| path.file_name_str().cmp(&Some(GITIGNORE_FILE)))
                    .ok()
                    .map(|idx| items[idx].clone())
                    .and_then(parse_gitignore);

                (Box::new(items.into_iter()), ignore)
            }
            Err(le) => {
                error!("swallowed list error : {:?}", le);
                (Box::new(std::iter::empty()), None)
            }
        };

        RecursiveFsIter {
            stack: Vec::from([first_iter]),
            ignore,
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

            if let Some(ref ignore) = self.ignore {
                if ignore.matched(item.absolute_path(), item.is_dir()).is_ignore() {
                    continue;
                }
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
    fn test_nested_gitignore_does_not_affect_other_files() {
        let m = MockFS::new("/tmp")
            .with_file("folder1/folder2/.gitignore", "file1.txt")
            .with_file("folder1/folder2/file1.txt", "not matched")
            .with_file("folder1/folder3/file1.txt", "matched")
            .to_fsf();

        let mut iter = RecursiveFsIter::new(m.root());

        assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "file1.txt").unwrap()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic]
    fn test_multiple_gitignores_are_respected() {
        let m = MockFS::new("/tmp")
            .with_file(".gitignore", "file1.txt")
            .with_file("folder1/folder2/.gitignore", "file2.txt")
            .with_file("folder1/folder2/file1.txt", "not matched")
            .with_file("folder1/folder2/file2.txt", "not matched")
            .with_file("folder1/folder3/moulder.txt", "truth is out there")
            .with_file("folder1/folder3/file1.txt", "not matched")
            .with_file("folder1/folder3/file2.txt", "matched")
            .to_fsf();

        let mut iter = RecursiveFsIter::new(m.root());

        assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
        assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "file2.txt").unwrap()));
        assert_eq!(iter.next(), None);
    }
}
