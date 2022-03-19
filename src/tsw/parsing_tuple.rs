use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use tree_sitter::{Language, Parser, Query, Tree};
use crate::tsw::lang_id::LangId;

#[derive(Clone)]
pub struct ParsingTuple {
    // none before first parse
    pub tree: Option<Tree>,

    pub lang_id: LangId,
    pub parser: Rc<RefCell<Parser>>,
    pub language: Language,
    pub highlight_query: Rc<Query>,
    pub id_to_name: Rc<Vec<Rc<String>>>,
}

impl Debug for ParsingTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "lang_id {:?}", self.lang_id)
    }
}

