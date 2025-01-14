use log::debug;

use crate::experiments::focus_group::FocusUpdate;
use crate::experiments::screen_shot::screenshot;
use crate::io::input_event::InputEvent;
use crate::io::keys::{Key, Keycode};
use crate::mocks::full_setup::FullSetup;
use crate::mocks::treeview_interpreter::{TreeViewInterpreter, TreeViewInterpreterItem};
use crate::mocks::with_wait_for::WithWaitFor;
use crate::spath;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_file_dialog_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save_as));

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().save_file_dialog().is_some()));

    assert!(full_setup.get_first_editor().unwrap().save_file_dialog().unwrap().is_focused());

    let expanded_items: Vec<(String, bool)> = tree_items(&full_setup)
        .iter()
        .map(|item| (item.label.clone(), item.expanded))
        .collect::<Vec<_>>();
    assert_eq!(
        expanded_items,
        vec![("save_file_dialog_test_1".to_string(), true), ("src".to_string(), false)]
    );

    full_setup
}

fn tree_items(full_setup: &FullSetup) -> Vec<TreeViewInterpreterItem> {
    full_setup
        .get_first_editor()
        .unwrap()
        .save_file_dialog()
        .unwrap()
        .tree_view()
        .items()
}

#[test]
fn path_expanded() {
    let full_setup = common_start();

    let expanded: Vec<String> = full_setup
        .get_first_editor()
        .unwrap()
        .save_file_dialog()
        .unwrap()
        .tree_view()
        .items()
        .into_iter()
        .filter(|item| item.expanded)
        .map(|item| item.label)
        .collect();

    assert_eq!(expanded, vec!["save_file_dialog_test_1".to_string()]);

    full_setup.finish();
}

#[test]
fn no_leak_focus() {
    // this test validates, that when save-dialog is open, editor cannot be modified, but tree view can.

    let mut full_setup = common_start();

    assert!(!full_setup.get_first_editor().unwrap().is_editor_focused());

    full_setup.send_input(InputEvent::FocusUpdate(FocusUpdate::Left));

    assert!(!full_setup.get_first_editor().unwrap().is_editor_focused());

    assert!(full_setup.wait_for(|f| { f.get_file_tree_view().unwrap().is_focused() }));

    full_setup.finish();
}

#[test]
fn expanded_and_highlighted_path() {
    let full_setup = common_start();

    assert_eq!(
        tree_items(&full_setup)
            .iter()
            .filter(|i| i.expanded)
            .map(|i| i.label.clone())
            .collect::<Vec<_>>(),
        vec!["save_file_dialog_test_1"]
    );

    let src = tree_items(&full_setup).iter().find(|i| i.label == "src").unwrap().clone();

    assert!(!src.expanded);
    assert_eq!(src.depth, 1);
    assert!(src.highlighted);

    full_setup.finish();
}

#[test]
fn hit_on_dir_expands_it() {
    let mut full_setup = common_start();

    assert!(full_setup
        .get_first_editor()
        .unwrap()
        .save_file_dialog()
        .unwrap()
        .tree_view()
        .is_focused());

    assert!(
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .tree_view()
            .selected()
            .unwrap()
            .label
            == "src"
    );

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        tree_items(full_setup)
            .iter()
            .filter(|i| i.expanded)
            .map(|i| i.label.clone())
            .collect::<Vec<_>>()
            == vec!["save_file_dialog_test_1".to_string(), "src".to_string()]
    }));

    full_setup.finish();
}

#[test]
fn hit_on_leaf_dir_moves_focus() {
    let mut full_setup = common_start();

    assert!(full_setup
        .get_first_editor()
        .unwrap()
        .save_file_dialog()
        .unwrap()
        .tree_view()
        .is_focused());

    full_setup.send_key(Keycode::Enter.to_key());
    full_setup.send_key(Keycode::ArrowDown.to_key());
    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        !full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .tree_view()
            .is_focused()
    }));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .list_view()
            .is_focused()
    }));

    full_setup.finish();
}

#[test]
fn hit_on_list_item_moves_to_edit() {
    let mut full_setup = common_start();

    assert!(full_setup
        .get_first_editor()
        .unwrap()
        .save_file_dialog()
        .unwrap()
        .tree_view()
        .is_focused());

    full_setup.send_key(Keycode::ArrowRight.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .list_view()
            .is_focused()
    }));

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .edit_widget()
            .is_focused()
    }));

    full_setup.finish();
}

fn over_ok() -> FullSetup {
    let mut full_setup = common_start();

    assert!(full_setup
        .get_first_editor()
        .unwrap()
        .save_file_dialog()
        .unwrap()
        .tree_view()
        .is_focused());

    full_setup.send_key(Keycode::ArrowRight.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .list_view()
            .is_focused()
    }));

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .edit_widget()
            .is_focused()
    }));

    assert_eq!(
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .edit_widget()
            .contents(),
        "main.rs"
    );

    full_setup.send_key(Keycode::ArrowLeft.to_key());
    full_setup.send_key(Keycode::ArrowLeft.to_key());
    full_setup.send_key(Keycode::ArrowLeft.to_key());
    full_setup.send_key(Keycode::Char('2').to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .edit_widget()
            .contents()
            == "main2.rs"
    }));

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .ok_button()
            .is_focused()
    }));

    full_setup
}

#[test]
fn happy_path() {
    let mut full_setup = over_ok();

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        spath!(full_setup.fsf(), "src", "main2.rs")
            .map(|path| path.is_file())
            .unwrap_or(false)
    }));

    full_setup.finish();
}

#[test]
fn esc_cancels_path() {
    let mut full_setup = common_start();

    full_setup.send_key(Keycode::Esc.to_key());

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_editor().unwrap().save_file_dialog().is_none() }));

    full_setup.finish();
}

#[test]
fn cancel_cancels() {
    let mut full_setup = over_ok();

    full_setup.send_key(Keycode::ArrowLeft.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .save_file_dialog()
            .unwrap()
            .cancel_button()
            .is_focused()
    }));

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_editor().unwrap().save_file_dialog().is_none() }));

    full_setup.finish();
}

#[test]
fn save_empty_file_doesnt_leak_focus() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_file_dialog_test_1").build();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.new_buffer));
    assert!(full_setup.wait_for(|full_setup| full_setup.get_first_editor().is_some()));
    assert!(full_setup.get_first_editor().unwrap().is_view_focused());
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save));
    assert!(full_setup.wait_for(|full_setup| full_setup.get_first_editor().unwrap().save_file_dialog().is_some()));

    assert!(full_setup.wait_for(|full_setup| full_setup.get_first_editor().unwrap().save_file_dialog().unwrap().is_focused()));
}
