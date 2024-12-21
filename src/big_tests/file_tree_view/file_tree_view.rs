use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_tree_view_test_1").build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));
    assert!(full_setup.send_key(Keycode::ArrowLeft.to_key().with_alt()));

    assert!(full_setup.wait_for(|f| f.get_file_tree_view().unwrap().is_focused()));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|f| f.get_file_tree_view().unwrap().items().len() > 1));

    full_setup
}

// fn tree_items(full_setup: &FullSetup) -> Vec<TreeViewInterpreterItem> {
//     full_setup
//         .get_first_editor()
//         .unwrap()
//         .save_file_dialog()
//         .unwrap()
//         .tree_view()
//         .items()
// }

#[test]
fn file_tree_opens() {
    let mut full_setup = common_start();

    for _ in 0..2 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
    }

    assert!(full_setup.wait_for(|f| {
        f.get_file_tree_view()
            .unwrap()
            .items()
            .iter()
            .find(|item| item.highlighted)
            .unwrap()
            .label
            .starts_with("chapter3.txt")
    }));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|f| { f.is_editor_opened() }));
}

#[test]
fn toggle_filter() {
    let mut full_setup = common_start();

    let original_count = full_setup.get_file_tree_view().unwrap().items().len();
    assert_eq!(original_count, 5);
    assert!(full_setup
        .get_file_tree_view()
        .unwrap()
        .items()
        .iter()
        .find(|item| item.label.starts_with(".gladius"))
        .is_some());

    assert!(full_setup.send_key(full_setup.config().keyboard_config.file_tree.toggle_hidden_files));

    assert!(full_setup.wait_for(|full_setup| full_setup
        .get_file_tree_view()
        .unwrap()
        .items()
        .iter()
        .find(|item| item.label.starts_with(".gladius"))
        .is_none()));

    let new_count = full_setup.get_file_tree_view().unwrap().items().len();
    assert!(new_count < original_count);
    assert_eq!(new_count, 4);
}
