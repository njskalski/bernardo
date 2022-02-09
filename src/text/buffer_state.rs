use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;
use std::string::String;

use log::{debug, error, warn};
use ropey::iter::Lines;
use ropey::Rope;
use tree_sitter::{Parser, Point, Tree, TreeCursor};
use unicode_segmentation::UnicodeSegmentation;

use crate::{Theme, TreeSitterWrapper};
use crate::experiments::tree_sitter_wrapper::LangId;
use crate::text::buffer::Buffer;

#[derive(Clone)]
struct ParsingTuple {
    pub tree: Tree,
    pub lang_id: LangId,
    pub parser: Rc<Parser>,
}

impl Debug for ParsingTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "lang_id {:?}", self.lang_id)
    }
}

#[derive(Clone, Debug, Default)]
struct Text {
    pub rope: Rope,
    pub parsing: Option<ParsingTuple>,
}

impl Text {
    pub fn with_rope(self, rope: Rope) -> Self {
        Self {
            rope,
            ..self
        }
    }

    pub fn parse(&mut self, tree_sitter: Rc<TreeSitterWrapper>, lang_id: LangId) -> bool {
        if let Some((parser, tree)) = tree_sitter.new_parse(lang_id, &self.rope) {
            self.parsing = Some(ParsingTuple {
                tree,
                lang_id,
                parser: Rc::new(parser),
            });
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct BufferState {
    tree_sitter: Rc<TreeSitterWrapper>,

    text: Text,

    history: Vec<Text>,
    forward_history: Vec<Text>,

    lang_id: Option<LangId>,

    file_path: Option<PathBuf>,
}

impl BufferState {
    pub fn new(tree_sitter: Rc<TreeSitterWrapper>) -> BufferState {
        BufferState {
            tree_sitter,
            text: Text::default(),
            history: vec![],
            forward_history: vec![],

            file_path: None,
            lang_id: None,
        }
    }

    pub fn with_lang(self, lang_id: LangId) -> Self {
        Self {
            lang_id: Some(lang_id),
            ..self
        }
    }

    pub fn with_text_from_rope(self, rope: Rope, lang_id: Option<LangId>) -> Self {
        let mut text = Text::default().with_rope(rope);

        if let Some(lang_id) = lang_id {
            text.parse(self.tree_sitter.clone(), lang_id);
        }

        Self {
            text,
            ..self
        }
    }

    pub fn set_lang(&mut self, lang_id: Option<LangId>) {
        self.lang_id = lang_id;
    }

    fn clone_top(&mut self) {
        self.history.push(self.text.clone());
    }

    fn after_change(&mut self) {
        self.forward_history.clear();
    }

    pub fn prev(&mut self) -> bool {
        match self.history.pop() {
            None => false,
            Some(r) => {
                self.forward_history.push(self.text.clone());
                self.text = r;
                true
            }
        }
    }

    pub fn next(&mut self) -> bool {
        match self.forward_history.pop() {
            None => false,
            Some(r) => {
                self.history.push(self.text.clone());
                self.text = r;
                true
            }
        }
    }

    pub fn char_to_kind(&self, char_idx: usize) -> Option<&str> {
        // let byte_idx = match self.text.try_char_to_byte(char_idx) {
        //     Ok(idx) => idx,
        //     _ => return None,
        // };

        // self.todo_parse_tree.as_ref().map(|tree| {
        //     tree.root_node().descendant_for_byte_range(byte_idx, byte_idx)
        // }).flatten().map(|node| {
        //     node.kind()
        // })
        None
    }
}

impl Buffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text.rope.len_lines()
    }
    fn lines(&self) -> Box<dyn Iterator<Item=String> + '_> {
        // TODO this will fail for large files
        Box::new(self.text.rope.lines().map(|f| f.to_string()))
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_chars(&self) -> usize {
        self.text.rope.len_chars()
    }

    fn char_to_line(&self, char_idx: usize) -> Option<usize> {
        match self.text.rope.try_char_to_line(char_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn line_to_char(&self, line_idx: usize) -> Option<usize> {
        match self.text.rope.try_line_to_char(line_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool {
        match self.text.rope.try_insert_char(char_idx, ch) {
            Ok(_) => true,
            Err(e) => {
                warn!("failed inserting char {} because {}", char_idx, e);
                false
            }
        }
    }

    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if !(char_idx_end > char_idx_begin) {
            error!("requested removal of improper range ({}, {})", char_idx_begin, char_idx_end);
            return false;
        }

        match self.text.rope.try_remove(char_idx_begin..char_idx_end) {
            Ok(_) => true,
            Err(e) => {
                warn!("failed removing char {:?}-{:?} because {}", char_idx_begin, char_idx_end, e);
                false
            }
        }
    }

    fn char_at(&self, char_idx: usize) -> Option<char> {
        self.text.rope.char_at(char_idx)
    }

    fn callback_for_parser<'a>(&'a self) -> Box<dyn FnMut(usize, Point) -> &'a [u8] + 'a> {
        self.text.rope.callback_for_parser()
    }
}