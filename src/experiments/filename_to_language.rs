use std::collections::HashMap;
use std::path::Path;

use crate::experiments::tree_sitter_wrapper::LangId;

lazy_static! {
    static ref EXT_TO_LANGUAGE : HashMap<&'static str, LangId> = hashmap! {
        "htm" => LangId::HTML,
        "html" => LangId::HTML,
        "c" => LangId::C,
        "h" => LangId::C,
        // yeah, so C++ is such a mess, that even the filenames are not so simple.
        "cc" => LangId::CPP,
        "C" => LangId::CPP,
        "cpp" => LangId::CPP,
        "cxx" => LangId::CPP,
        "c++" => LangId::CPP,
        "cppm" => LangId::CPP, // even clion thinks this one is a typo
        "hxx" => LangId::CPP,
        "hpp" => LangId::CPP,
        "ixx" => LangId::CPP, // and this is just pure bs.

        // back to normal languages
        "elm" => LangId::ELM,
        "rs" => LangId::RUST,
    };
}

pub fn filename_to_language(path: &Path) -> Option<LangId> {
    path.extension().map(|ext| ext.to_str()).flatten().map(|ext|
        EXT_TO_LANGUAGE.get(ext).map(|p| *p)
    ).flatten()
}