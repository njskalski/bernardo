use crossterm::event::KeyCode;
use log::debug;

use crate::mocks::full_setup::FullSetup;

#[test]
fn save_file_dialog_happy_path() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_file_dialog_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save_as));

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().save_file_dialog().is_some()));

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


    full_setup.screenshot();
}