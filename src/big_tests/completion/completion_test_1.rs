use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::config::config::Config;
use crate::config::theme::Theme;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::gladius::run_gladius::run_gladius;
use crate::mocks::full_setup::FullSetupBuilder;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::mocks::mock_input::MockInput;
use crate::mocks::mock_output::MockOutput;
use crate::spath;

#[test]
fn completion_test_1() {
    let mut full_setup = FullSetupBuilder::new("./test_envs/completion_test_1")
        .with_files(["src/main.rs"])
        .build();


    let file = spath!(full_setup.fsf(), "src", "main.rs").unwrap();

    assert!(full_setup.wait_frame());
    assert!(full_setup.is_editor_opened());

    assert!(full_setup.navcomp_pilot().wait_for_load(&file).is_some());

    let end = full_setup.finish();
    assert!(end.screenshot());
}