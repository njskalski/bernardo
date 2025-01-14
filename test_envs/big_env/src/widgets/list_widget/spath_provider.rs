// use std::fmt::{Debug, Formatter};
//
// use log::error;
//
// use crate::fs::path::SPath;
// use crate::fs::read_error::ListError;
// use crate::widgets::list_widget::provider::ListItemProvider;
//
// #[derive(Debug)]
// pub struct SPathListItemProvider {
//     spath: SPath,
// }
//
// impl SPathListItemProvider {
//     pub fn new(spath: SPath) -> Self {
//         Self {
//             spath
//         }
//     }
// }
//
// impl ListItemProvider<SPath> for SPathListItemProvider {
//     fn iter(&self) -> Box<dyn Iterator<Item=&SPath> + '_> {
//         if self.spath.is_dir() {
//             match self.spath.blocking_list() {
//                 Ok(items) => {
//                     Box::new(items.into_iter().map(|d| &d))
//                 }
//                 Err(e) => {
//                     error!("failed listing {:?}", self.spath);
//                     Box::new(std::iter::empty())
//                 }
//             }
//         } else {
//             Box::new(std::iter::empty())
//         }
//     }
// }
//
//
// #[derive(Debug)]
// pub struct SPathListFilesItemProvider {
//     spath: SPath,
// }
//
// impl SPathListFilesItemProvider {
//     pub fn new(spath: SPath) -> Self {
//         Self {
//             spath
//         }
//     }
// }
//
// impl ListItemProvider<SPath> for SPathListFilesItemProvider {
//     fn iter(&self) -> Box<dyn Iterator<Item=&SPath> + '_> {
//         if self.spath.is_dir() {
//             match self.spath.blocking_list() {
//                 Ok(items) => {
//                     Box::new(items.into_iter().filter(|i| i.is_file()))
//                 }
//                 Err(e) => {
//                     error!("failed listing {:?}", self.spath);
//                     Box::new(std::iter::empty())
//                 }
//             }
//         } else {
//             Box::new(std::iter::empty())
//         }
//     }
// }
