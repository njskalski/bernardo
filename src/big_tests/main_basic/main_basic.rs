use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

#[test]
fn no_file_open_focuses_on_file_tree() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_basic_test_1").build();
    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));
    assert!(full_setup.get_file_tree_view().unwrap().is_focused());
}

#[test]
fn with_file_open_editor_is_focused() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_basic_test_1").with_files(["src/main.rs"]).build();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.get_first_editor().unwrap().is_editor_focused());
    assert_eq!(full_setup.get_file_tree_view().unwrap().is_focused(), false);
}
