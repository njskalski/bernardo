use std::path::PathBuf;

use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;

#[test]
fn completion_test_1() {
    let mock_fs = MockFS::generate_from_real("./test_envs/completion_test_1").unwrap();
    assert!(mock_fs.is_file(&PathBuf::from("src/main.rs")));


    println!("{:?}", mock_fs);
}