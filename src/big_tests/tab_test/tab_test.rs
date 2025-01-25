use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/tab_test_1")
        .with_files(["file_with_tabs.txt"].iter())
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup
}

#[test]
fn find_in_files_opens() {
    let mut f = common_start();

    let x: Vec<_> = f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines_with_coded_cursors()
        .collect();

    let x = x;
}
