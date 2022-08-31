// use std::path::{Component, Path, PathBuf};
//
// pub struct PathPrefixes<'a> {
//     path: &'a Path,
//     idx: usize,
// }
//
// impl<'a> PathPrefixes<'a> {
//     pub fn new(path: &'a Path) -> Self {
//         Self {
//             path,
//             idx: 0,
//         }
//     }
// }
//
// impl Iterator for PathPrefixes<'_> {
//     type Item = PathBuf;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.idx > self.path.components().count() {
//             return None;
//         }
//         self.idx += 1;
//
//         let res = self.path.components().take(self.idx).fold(PathBuf::new(),
//                                                              |acc: PathBuf, c: Component| {
//                                                                  acc.join(c)
//                                                              });
//
//         Some(res)
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use std::path::PathBuf;
//
//     use crate::experiments::path_prefixes::PathPrefixes;
//
//     #[test]
//     fn test_some_prefixes() {
//         let path = PathBuf::from("/some/path/file.txt");
//
//         let mut pp = PathPrefixes::new(path.as_path());
//
//         assert_eq!(pp.next(), Some(PathBuf::from("/")));
//         assert_eq!(pp.next(), Some(PathBuf::from("/some")));
//         assert_eq!(pp.next(), Some(PathBuf::from("/some/path")));
//         assert_eq!(pp.next(), None);
//     }
// }