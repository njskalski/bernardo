use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::config::config::Config;
use crate::config::theme::Theme;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::gladius::run_gladius::run_gladius;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::mocks::mock_input::MockInput;
use crate::mocks::mock_output::MockOutput;

#[test]
fn completion_test_1() {
    let mock_fs = MockFS::generate_from_real("./test_envs/completion_test_1").unwrap();
    assert!(mock_fs.is_file(&PathBuf::from("src/main.rs")));
}