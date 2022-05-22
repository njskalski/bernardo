use std::fmt::Debug;
use log::error;
use crate::fs::file_front::FileFront;
use crate::{AnyMsg, FsfRef};
use crate::experiments::beter_deref_str::BetterDerefStr;
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
    fn display_name(&self) -> BetterDerefStr {
        BetterDerefStr::Str(self.display_file_name())
    }

    fn comment(&self) -> Option<BetterDerefStr> {
        Some(BetterDerefStr::Str(self.display_last_dir_name(true)))
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