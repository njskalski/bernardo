use std::borrow::Borrow;
use std::iter;
use std::marker::PhantomData;
use std::rc::Rc;
use std::vec::IntoIter;
use log::error;
use crate::fs::file_front::FileFront;
use crate::{AnyMsg, FsfRef};
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

pub type FileFrontToMsg = fn(&FileFront) -> Box<dyn AnyMsg>;

// TODO add subdirectory
pub struct FsfProvider {
    fsf: FsfRef,
    msg_mapper: FileFrontToMsg,
}

pub struct WrappedFileFront<'a> {
    ff: FileFront,
    msg_mapper: &'a FileFrontToMsg,
}

impl<'a> WrappedFileFront<'a> {
    pub fn new(ff: FileFront, msg_mapper: &'a FileFrontToMsg) -> WrappedFileFront<'a> {
        WrappedFileFront {
            ff,
            msg_mapper,
        }
    }
}

impl<'a> Item for WrappedFileFront<'a> {
    fn display_name(&self) -> &str {
        self.ff.path().to_str().unwrap_or_else(|| {
            error!("failed to cast path to string: {:?}", self.ff.path());
            "failed cast"
        })
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        (self.msg_mapper)(&self.ff)
    }
}

// struct WrapIter<'a, T: 'a> {
//     iter: Box<dyn Iterator<Item=T>>,
//     _phantom: PhantomData<&'a T>,
// }
//
// impl<'a, T: 'a> WrapIter<'a, T> {
//     pub fn new(iter: Box<dyn Iterator<Item=T>>) -> Self {
//         Self {
//             iter,
//             _phantom: PhantomData::default(),
//         }
//     }
// }
//
// impl<'a, T> Iterator for WrapIter<'a, T> {
//     type Item = &'a T;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next().as_ref()
//     }
// }

// impl ItemsProvider for FsfProvider {
//     fn context_name(&self) -> &str {
//         "fs"
//     }
//
//     fn items<'a>(&'a self, query: String) -> Box<dyn Iterator<Item=&'a (dyn Item + 'a)> + '_> {
//         // TODO since I have no limit on iterator, at this point I just ignore queries below 4 letters.
//         if query.len() < 4 {
//             Box::new(iter::empty())
//         } else {
//             let items: Vec<WrappedFileFront> = self.fsf
//                 .fuzzy_files_it(query, 100).1
//                 .map(|item| WrappedFileFront::new(item, &self.msg_mapper))
//                 .collect(); // eeee makarena!
//
//             Box::new(WrapIter::new(Box::new(items.into_iter())).map(|f| f as &dyn Item))
//         }
//     }
// }