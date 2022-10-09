use crossterm::event::KeyCode;
use log::debug;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::input_event::InputEvent;
use crate::mocks::full_setup::FullSetup;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_file_dialog_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save_as));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().save_file_dialog().is_some()));

    full_setup
}

#[test]
fn path_expanded() {
    let mut full_setup = common_start();

    let expanded: Vec<String> = full_setup
        .get_first_editor().unwrap()
        .save_file_dialog().unwrap()
        .tree_view().items().into_iter().filter(|item| item.expanded)
        .map(|item| item.label)
        .collect();

    assert_eq!(expanded, vec![
        "save_file_dialog_test_1".to_string(),
        "src".to_string(),
    ]);
}

#[test]
fn no_leak_focus() {
    // this test validates, that when save-dialog is open, editor cannot be modified, but tree view can.

    let mut full_setup = common_start();

    assert_eq!(full_setup.get_first_editor().unwrap().is_editor_focused(), false);

    full_setup.send_input(InputEvent::FocusUpdate(FocusUpdate::Left));

    assert_eq!(full_setup.get_first_editor().unwrap().is_editor_focused(), false);

    assert!(full_setup.wait_for(|f| {
        f.get_file_tree_view().unwrap().is_focused()
    }));
}