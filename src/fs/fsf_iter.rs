use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::error;

use crate::fs::path::SPath;

use super::read_error::ListError;

fn parse_gitignore(ignore_path: SPath) -> Option<Gitignore> {
    let absolute_path = ignore_path.absolute_path();
    let contents = ignore_path.read_entire_file_to_string().ok()?;

    let mut builder = GitignoreBuilder::new(ignore_path.parent()?.absolute_path());
    for line in contents.lines() {
        // GitignoreBuilder::add() can't be used because it uses an File::open()
        // call on the filepath, but we use our own FS layer
        let _ = builder.add_line(Some(absolute_path.clone()), line);
    }

    builder
        .build()
        .inspect_err(|err| error!("could not build Gitignore for path {ignore_path}: {err}"))
        .ok()
}

const GITIGNORE_FILE: &str = ".gitignore";

/// Contents of a directory, sorted lexicographically. Also identifies a .gitignore
/// file if it exists in the directory.
struct DirContents {
    files: Box<dyn Iterator<Item = SPath>>,
    ignore: Option<Gitignore>,
}

impl DirContents {
    pub fn from_dir(dir: SPath) -> Result<Self, ListError> {
        let mut files: Vec<_> = dir.blocking_list()?.map(|i| i.clone()).collect();
        files.sort();

        let ignore = files
            // It should be fine to compare by the basename since items only contains
            // files from a single directory
            .binary_search_by(|path| path.file_name_str().cmp(&Some(GITIGNORE_FILE)))
            .ok()
            .map(|idx| files[idx].clone())
            .and_then(parse_gitignore);

        Ok(DirContents {
            files: Box::new(files.into_iter()),
            ignore,
        })
    }

    pub fn empty() -> Self {
        DirContents {
            files: Box::new(std::iter::empty()),
            ignore: None,
        }
    }
}

impl Default for DirContents {
    fn default() -> Self {
        Self::empty()
    }
}

/*
Recursively iterates over all items under root, in DFS pattern, siblings sorted lexicographically
 */
pub struct RecursiveFsIter {
    stack: Vec<DirContents>,
}

impl RecursiveFsIter {
    pub fn new(root: SPath) -> Self {
        let contents = DirContents::from_dir(root)
            .inspect_err(|le| error!("swallowed list error : {:?}", le))
            .unwrap_or_default();

        RecursiveFsIter { stack: vec![contents] }
    }

    /// Checks whether a path is ignored by comparing against all gitignore files
    /// in ancestor directories till the root directory.
    fn is_ignored(&self, path: &SPath) -> bool {
        self.stack
            .iter()
            .filter_map(|dir| dir.ignore.as_ref())
            .any(|ig| ig.matched(path.absolute_path(), path.is_dir()).is_ignore())
    }
}

impl Iterator for RecursiveFsIter {
    type Item = SPath;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(dir) = self.stack.last_mut() {
            let Some(file) = dir.files.next() else {
                // The dir has been completely iterated over; remove it
                self.stack.pop();
                continue;
            };

            if file.is_hidden() || self.is_ignored(&file) {
                continue;
            }

            if file.is_dir() {
                match DirContents::from_dir(file.clone()) {
                    Ok(contents) => self.stack.push(contents),
                    Err(le) => error!("swallowed list error 2 : {:?}", le),
                };
            }

            return Some(file);
        }

        None
    }
}
