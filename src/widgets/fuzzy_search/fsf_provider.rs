use std::borrow::Cow;
use std::fmt::Debug;
use std::io::empty;

use crate::{AnyMsg};
use crate::experiments::beter_deref_str::BetterDerefStr;
use crate::new_fs::fsf_ref::FsfRef;
use crate::new_fs::path::SPath;
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

pub type SPathToMsg = fn(&SPath) -> Box<dyn AnyMsg>;


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
pub enum SPathMsg {
    Hit(SPath),
}

impl AnyMsg for SPathMsg {}

impl Item for SPath {
    fn display_name(&self) -> Cow<str> {
        self.file_name_str().unwrap_or("<error>").into()
    }

    fn comment(&self) -> Option<Cow<str>> {
        Some(self.display_name())
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        Box::new(SPathMsg::Hit(self.clone()))
    }
}

impl ItemsProvider for FsfProvider {
    fn context_name(&self) -> &str {
        "fs"
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        // let items = self.fsf.fuzzy_files_it(query, limit, self.consider_ignores).1.map(|f| Box::new(f) as Box<dyn Item>);
        Box::new(std::iter::empty())
    }
}