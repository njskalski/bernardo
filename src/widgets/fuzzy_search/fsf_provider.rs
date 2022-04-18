use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::marker::PhantomData;
use std::rc::Rc;
use std::vec::IntoIter;
use log::{debug, error};
use crate::fs::file_front::FileFront;
use crate::{AnyMsg, FsfRef};
use crate::fs::constants::{NON_UTF8_ERROR_STR, NOT_A_FILENAME, OUTSIDE_ROOT};
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

pub type FileFrontToMsg = fn(&FileFront) -> Box<dyn AnyMsg>;


// TODO add subdirectory
pub struct FsfProvider {
    fsf: FsfRef,
    consider_ignores: bool,
}

impl FsfProvider {
    pub fn new(fsf: FsfRef) -> Self {
        Self {
            fsf,
            consider_ignores: false,
        }
    }

    pub fn with_ignores_filter(self) -> Self {
        Self {
            consider_ignores: true,
            ..self
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
        self.path().file_name().map(|oss| oss.to_str().unwrap_or_else(|| {
            error!("failed to cast path to string: {:?}", self.path());
            NON_UTF8_ERROR_STR
        })).unwrap_or_else(|| {
            error!("failed to extract a filename from: {:?}", self.path());
            NOT_A_FILENAME
        })
    }

    fn comment(&self) -> Option<&str> {
        self.path().strip_prefix(self.fsf().get_root_path().as_path()).map(
            |stripped_path| {
                stripped_path.to_str().unwrap_or_else(|| {
                    error!("failed to cast path to string: {:?}", stripped_path);
                    NON_UTF8_ERROR_STR
                })
            }
        ).map_err(|e| {
            error!("Strip prefix error {}. File {:?} outside root?", e, self.path());
            OUTSIDE_ROOT
        }).ok()
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        Box::new(FileFrontMsg::Hit(self.clone()))
    }
}

impl ItemsProvider for FsfProvider {
    fn context_name(&self) -> &str {
        "fs"
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        let items = self.fsf.fuzzy_files_it(query, limit, self.consider_ignores).1.map(|f| Box::new(f) as Box<dyn Item>);
        Box::new(items)
    }
}