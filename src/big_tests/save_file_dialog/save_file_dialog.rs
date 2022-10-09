use log::debug;
use lsp_types::lsif::Item;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::treeview_interpreter::TreeViewInterpreterItem;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_file_dialog_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save_as));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().save_file_dialog().is_some()));

    full_setup
}

fn tree_items(full_setup: &FullSetup) -> Vec<TreeViewInterpreterItem> {
    full_setup.get_first_editor().unwrap()
        .save_file_dialog().unwrap()
        .tree_view().items()
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

#[test]
fn expanded_and_highlighted_path() {
    // this test validates, that when save-dialog is open, editor cannot be modified, but tree view can.

    let mut full_setup = common_start();


    assert_eq!(tree_items(&full_setup).iter()
                   .filter(|i| i.expanded)
                   .map(|i| i.label.clone())
                   .collect::<Vec<_>>(), vec!["save_file_dialog_test_1"]);

    let src = tree_items(&full_setup).iter().find(|i| i.label == "src").unwrap().clone();

    assert_eq!(src.expanded, false);
    assert_eq!(src.depth, 1);
    assert_eq!(src.highlighted, true);
}

#[test]
fn hit_on_dir_expands_it() {
    // this test validates, that when save-dialog is open, editor cannot be modified, but tree view can.

    let mut full_setup = common_start();

    assert_eq!(full_setup.get_first_editor().unwrap()
                   .save_file_dialog().unwrap()
                   .tree_view().is_focused(), true
    );

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        tree_items(&full_setup).iter()
            .filter(|i| i.expanded)
            .map(|i| i.label.clone())
            .collect::<Vec<_>>() == vec!["save_file_dialog_test_1".to_string(), "src".to_string()]
    }));
}

#[test]
fn hit_on_leaf_dir_moves_focus() {
    // this test validates, that when save-dialog is open, editor cannot be modified, but tree view can.

    let mut full_setup = common_start();

    assert_eq!(full_setup.get_first_editor().unwrap()
                   .save_file_dialog().unwrap()
                   .tree_view().is_focused(), true
    );

    full_setup.send_key(Keycode::Enter.to_key());
    full_setup.send_key(Keycode::ArrowDown.to_key());
    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().save_file_dialog().unwrap().tree_view().is_focused() == false
    }));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().save_file_dialog().unwrap().list_view().is_focused()
    }));

    full_setup.screenshot();
}

#[test]
fn hit_on_list_item_moves_to_edit() {
    // this test validates, that when save-dialog is open, editor cannot be modified, but tree view can.

    let mut full_setup = common_start();

    assert_eq!(full_setup.get_first_editor().unwrap()
                   .save_file_dialog().unwrap()
                   .tree_view().is_focused(), true
    );

    full_setup.send_key(Keycode::Enter.to_key());
    full_setup.send_key(Keycode::ArrowDown.to_key());
    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().save_file_dialog().unwrap().tree_view().is_focused() == false
    }));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().save_file_dialog().unwrap().list_view().is_focused()
    }));

    full_setup.screenshot();
}