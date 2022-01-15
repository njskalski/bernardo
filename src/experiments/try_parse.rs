use log::{error, warn};
use tree_sitter::{Language, LanguageError, Parser, Tree};

extern "C" { fn tree_sitter_rust() -> Language; }

pub fn try_parsing_rust(s: &str) -> Option<tree_sitter::Tree> {
    let language_rust = unsafe { tree_sitter_rust() };
    let mut parser = Parser::new();

    match parser.set_language(language_rust) {
        Err(err) => {
            error!("failed setting parser language: {}", err);
            return None;
        }
        Ok(_) => {}
    };

    parser.parse(s, None)
}