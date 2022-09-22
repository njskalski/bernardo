use std::borrow::Cow;
use std::fmt::Debug;

use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::widget::any_msg::AnyMsg;
use crate::widgets::fuzzy_search::helpers::is_subsequence;
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
        self.parent_ref().map(|p| p.relative_path().to_string_lossy().to_string().into())
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        Box::new(SPathMsg::Hit(self.clone()))
    }
}

// TODO reintroduce ignoring of gitignores
impl ItemsProvider for FsfProvider {
    fn context_name(&self) -> &str {
        "fs"
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        Box::new(
            self.fsf.root()
                .recursive_iter()
                .filter(
                    move |item| {
                        let item_str = item.relative_path().to_string_lossy().to_string();
                        is_subsequence(&item_str, &query)
                    }
                )
                .map(|f| Box::new(f) as Box<dyn Item>)
                .take(limit)
        )
    }
}