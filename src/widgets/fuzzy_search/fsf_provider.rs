use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::marker::PhantomData;
use std::rc::Rc;
use std::vec::IntoIter;
use log::{debug, error};
use crate::fs::file_front::FileFront;
use crate::{AnyMsg, FsfRef};
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

pub type FileFrontToMsg = fn(&FileFront) -> Box<dyn AnyMsg>;

// TODO add subdirectory
pub struct FsfProvider {
    fsf: FsfRef,
}

impl FsfProvider {
    pub fn new(fsf: FsfRef) -> Self {
        Self {
            fsf
        }
    }
}

#[derive(Debug)]
pub enum FileFrontMsg {
    Hit(FileFront),
}

impl AnyMsg for FileFrontMsg {}

impl Item for FileFront {
    fn display_name(&self) -> &str {
        self.path().to_str().unwrap_or_else(|| {
            error!("failed to cast path to string: {:?}", self.path());
            "failed cast"
        })
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        Box::new(FileFrontMsg::Hit(self.clone()))
    }
}

impl ItemsProvider for FsfProvider {
    fn context_name(&self) -> &str {
        "fs"
    }

    fn items(&self, query: String) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        // TODO since I have no limit on iterator, at this point I just ignore queries below 4 letters.
        // if query.len() < 4 {
        //     Box::new(iter::empty())
        // } else {
        let items = self.fsf.fuzzy_files_it(query, 100).1.map(|f| Box::new(f) as Box<dyn Item>);
        Box::new(items)
        // }
    }
}