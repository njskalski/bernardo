use std::thread;
use std::time::Duration;

use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn get_full_setup() -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/lsp_rust_integ_labels_1")
        .with_files(["src/main.rs"])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn rust_lsp_labels_test_1() {
    // if std::env::var("CI").is_ok() {
    //     return;
    // }

    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // TODO this should be replaced with "waiting for LSP to be ready", when some kind of statusbar
    // is implemented to signal presence of NavComp
    thread::sleep(Duration::from_secs(6));

    assert!(full_setup.wait_for(|full_setup| full_setup
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .find(|line| line.visible_idx == 1)
        .is_some()));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_editor().unwrap().get_errors().next().is_some() }));
}
