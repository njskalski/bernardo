use std::collections::HashMap;

use lazy_static::lazy_static;
use maplit::hashmap;

use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;

lazy_static! {
    static ref EXT_TO_LANGUAGE : HashMap<&'static str, LangId> = hashmap! {
        "sh" => LangId::BASH,

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

        "go" => LangId::GO,

        "hs" => LangId::HASKELL,

        // "htm" => LangId::HTML,
        // "html" => LangId::HTML,

        "java" => LangId::JAVA,

        "cjs" => LangId::JAVASCRIPT,
        "js" => LangId::JAVASCRIPT,
        "mjs" => LangId::JAVASCRIPT,

        "toml" => LangId::TOML,

        "ts" => LangId::TYPESCRIPT,

        "py" => LangId::PYTHON3,

        "rs" => LangId::RUST,

        "yaml" => LangId::YAML,
        "yml" => LangId::YAML,
    };
}

pub fn filename_to_language(path: &SPath) -> Option<LangId> {
    path.last_file_name()
        .map(|f| f.extension())
        .flatten()
        .map(|ext| ext.to_str())
        .flatten()
        .map(|ext| EXT_TO_LANGUAGE.get(ext).map(|p| *p))
        .flatten()
}
