use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    build_c();
    build_cpp();
    build_elm();
    build_go();
    build_html();
    build_rust();
}

fn build_c() {
    let dir: PathBuf = ["third-party", "tree-sitter-c", "src"].iter().collect();

    println!("cargo:rerun-if-changed=third-party/tree-sitter-c/src/parser.c");

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree-sitter-c");
}

fn build_cpp() {
    let dir: PathBuf = ["third-party", "tree-sitter-cpp", "src"].iter().collect();

    println!("cargo:rerun-if-changed=third-party/tree-sitter-cpp/src/parser.c");
    println!("cargo:rerun-if-changed=third-party/tree-sitter-cpp/src/scanner.cc");

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree-sitter-cpp-parser");

    cc::Build::new()
        .cpp(true)
        .include(&dir)
        .file(dir.join("scanner.cc"))
        .compile("tree-sitter-cpp-scanner");
}

fn build_elm() {
    let dir: PathBuf = ["third-party", "tree-sitter-elm", "src"].iter().collect();

    println!("cargo:rerun-if-changed=third-party/tree-sitter-elm/src/parser.c");
    println!("cargo:rerun-if-changed=third-party/tree-sitter-elm/src/scanner.cc");

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree-sitter-elm-parser");

    cc::Build::new()
        .cpp(true)
        .include(&dir)
        .file(dir.join("scanner.cc"))
        .compile("tree-sitter-elm-scanner");
}

fn build_go() {
    let dir: PathBuf = ["third-party", "tree-sitter-go", "src"].iter().collect();

    println!("cargo:rerun-if-changed=third-party/tree-sitter-elm/src/parser.c");

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree-sitter-go");
}

fn build_html() {
    let dir: PathBuf = ["third-party", "tree-sitter-html", "src"].iter().collect();

    println!("cargo:rerun-if-changed=third-party/tree-sitter-html/src/parser.c");

    println!("cargo:rerun-if-changed=third-party/tree-sitter-elm/src/parser.c");
    println!("cargo:rerun-if-changed=third-party/tree-sitter-elm/src/scanner.cc");

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree-sitter-html-parser");

    cc::Build::new()
        .cpp(true)
        .include(&dir)
        .file(dir.join("scanner.cc"))
        .compile("tree-sitter-html-scanner");
}

fn build_rust() {
    let dir: PathBuf = ["third-party", "tree-sitter-rust", "src"].iter().collect();

    println!("cargo:rerun-if-changed=third-party/tree-sitter-rust/src/parser.c");
    println!("cargo:rerun-if-changed=third-party/tree-sitter-rust/src/scanner.c");

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile("tree-sitter-rust");
}