use std::{
    env,
    fs,
    path::Path,
    path::PathBuf,

};


fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Tell Cargo to rerun the build script if anything in the "third-party" directory changes
    println!("cargo:rerun-if-changed=third-party");

    let grammars : Vec<(&'static str, &'static str, &'static str)> = vec![
        ("bash", "third-party/tree-sitter-bash", ""),
        ("c", "third-party/tree-sitter-c", ""),
        ("cpp", "third-party/tree-sitter-cpp", ""),
        ("haskell", "third-party/tree-sitter-haskell", ""),
        ("html", "third-party/tree-sitter-html", ""),
        ("java", "third-party/tree-sitter-java", ""),
        ("javascript", "third-party/tree-sitter-javascript", ""),
        ("typescript", "third-party/tree-sitter-typescript", ""),
        ("go", "third-party/tree-sitter-go", ""),
        ("python", "third-party/tree-sitter-python", ""),
        ("rust", "third-party/tree-sitter-rust", ""),
        ("toml", "third-party/tree-sitter-toml", "")
    ];

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());;

    for (name, path, _) in grammars {
        let path = manifest_dir.join(path);
        compile_tree_sitter_parser(&path, name.to_string());
    }
}


/// Compile the tree-sitter parser at the given path
fn compile_tree_sitter_parser(path: &Path, name : String) {

    println!("Compiling parser in: {:?}", path);

    // Example using tree_sitter_loader - adapt this to how you're actually using it
    let loader = tree_sitter_loader::Loader::new().unwrap();

    let lib_name = format!("tree-sitter-{}", name);
    let target_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    loader.compile_parser_at_path(path, target_path.clone(), &[]).unwrap();

    // Tell Cargo to link with the generated object file
    println!("cargo:rustc-link-lib=static={}", lib_name );
    println!("cargo:libdir={}", target_path.to_str().unwrap());
}
