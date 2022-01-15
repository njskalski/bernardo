use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["third-party", "tree-sitter-rust", "src"].iter().collect();

    println!("cargo:rerun-if-changed=third-party/tree-sitter-rust/src/parser.c");
    println!("cargo:rerun-if-changed=third-party/tree-sitter-rust/src/scanner.c");

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile("tree-sitter-rust");
}