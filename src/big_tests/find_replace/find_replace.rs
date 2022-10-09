use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_file_dialog_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.find));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().is_some()));
    assert!(full_setup.get_first_editor().unwrap().replace_op().is_none());

    full_setup
}


#[test]
fn find_shows_up() {
    let mut full_setup = common_start();
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().is_some()));

    full_setup.finish();
}

#[test]
fn find_is_focused() {
    let mut full_setup = common_start();
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().unwrap().is_focused()));
    full_setup.finish();
}

#[test]
fn esc_closes_find() {
    let mut full_setup = common_start();
    full_setup.send_key(Keycode::Esc.to_key());
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().is_none()));
    assert!(full_setup.get_first_editor().unwrap().is_view_focused());
    full_setup.finish();
}

#[test]
fn opening_replace_moves_focus() {
    let mut full_setup = common_start();
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().unwrap().is_focused()));

    full_setup.send_key(full_setup.config().keyboard_config.editor.replace);

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().replace_op().is_some()));

    assert!(full_setup.get_first_editor().unwrap().replace_op().unwrap().is_focused());
    assert_eq!(full_setup.get_first_editor().unwrap().find_op().unwrap().is_focused(), false);
    full_setup.finish();
}

#[test]
fn opening_replace_retains_contents_of_find() {
    let mut full_setup = common_start();
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().unwrap().is_focused()));

    full_setup.type_in("twoja stara");

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().unwrap().contents().contains("twoja stara")));

    full_setup.send_key(full_setup.config().keyboard_config.editor.replace);

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().replace_op().is_some()));

    assert!(full_setup.get_first_editor().unwrap().find_op().unwrap().contents().contains("twoja stara"));
    full_setup.finish();
}

#[test]
fn esc_closes_both() {
    let mut full_setup = common_start();
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().unwrap().is_focused()));

    full_setup.send_key(full_setup.config().keyboard_config.editor.replace);

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().replace_op().is_some()));

    full_setup.send_key(Keycode::Esc.to_key());
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().is_none()));
    assert!(full_setup.get_first_editor().unwrap().is_view_focused());
    full_setup.finish();
}