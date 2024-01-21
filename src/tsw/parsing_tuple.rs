use std::fmt::{Debug, Formatter};

use std::sync::{Arc, RwLock};

use tree_sitter::{Language, Parser, Query, Tree};

use crate::tsw::lang_id::LangId;

#[derive(Clone)]
pub struct ParsingTuple {
    // none before first parse
    pub tree: Option<Tree>,

    pub lang_id: LangId,
    pub parser: Arc<RwLock<Parser>>,
    pub language: Language,
    pub highlight_query: Arc<Query>,
    pub id_to_name: Arc<Vec<Arc<String>>>,
}

impl Debug for ParsingTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "lang_id {:?}", self.lang_id)
    }
}
