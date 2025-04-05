use std::{env, fs, path::Path, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Tell Cargo to rerun the build script if anything in the "third-party" directory changes
    println!("cargo:rerun-if-changed=third-party");

    let grammars: Vec<(&'static str, &'static str, &'static str)> = vec![
        ("bash", "third-party/tree-sitter-bash", ""),
        ("c", "third-party/tree-sitter-c", ""),
        ("cpp", "third-party/tree-sitter-cpp", ""),
        ("haskell", "third-party/tree-sitter-haskell", ""),
        ("html", "third-party/tree-sitter-html", ""),
        ("java", "third-party/tree-sitter-java", ""),
        ("javascript", "third-party/tree-sitter-javascript", ""),
        ("typescript", "third-party/tree-sitter-typescript", "typescript"),
        ("go", "third-party/tree-sitter-go", ""),
        ("python", "third-party/tree-sitter-python", ""),
        ("rust", "third-party/tree-sitter-rust", ""),
        ("toml", "third-party/tree-sitter-toml", ""),
        ("yaml", "third-party/tree-sitter-yaml", ""),
    ];

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    for (name, path, suffix) in grammars {
        let mut path = manifest_dir.join(path);
        if !suffix.is_empty() {
            path = path.join(suffix);
        }

        compile_tree_sitter_parser(&path, name.to_string());
    }
}

/// Compile the tree-sitter parser at the given path
fn compile_tree_sitter_parser(path: &Path, name: String) {
    println!("Compiling parser in: {:?}", path);

    let src_dir = path.join("src");
    let mut c_config = cc::Build::new();
    c_config.std("c11").include(&src_dir);

    let parser_path = src_dir.join("parser.c");
    if parser_path.exists() {
        c_config.file(&parser_path);
    }
    let scanner_path = src_dir.join("scanner.c");
    if scanner_path.exists() {
        c_config.file(&scanner_path);
    }

    println!("cargo:rerun-if-changed={}", parser_path.to_str().unwrap());

    let libname = format!("tree-sitter-{}", name);

    c_config.compile(&libname);

    println!("cargo:rustc-link-lib=static={}", &libname);
}
