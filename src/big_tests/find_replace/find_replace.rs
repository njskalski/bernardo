use log::debug;

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

// TODO(nj) add notifier indicating end-of-file
#[test]
fn actual_find() {
    let mut full_setup = common_start();
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().find_op().unwrap().is_focused()));

    full_setup.type_in("path");
    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().get_visible_cursor_lines_with_coded_cursors().find(|line| {
            debug!("line [{}]", line.contents.text);
            line.contents.text.contains("::(path]")
        }).is_some()
    }));

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().get_visible_cursor_lines_with_coded_cursors().find(|line| {
            debug!("line [{}]", line.contents.text);
            line.contents.text.contains("let (path] =")
        }).is_some()
    }));

    full_setup.finish();
}

#[test]
fn actual_replace() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_file_dialog_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.find));
    assert!(full_setup.type_in("path"));
    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().find_op().map(|find_op| find_op.contents().contains("path")).unwrap_or(false)
    }));

    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.replace));

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().replace_op().is_some()));

    assert!(full_setup.get_first_editor().unwrap().replace_op().is_some());

    assert!(full_setup.type_in("wrath"));
    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().replace_op().map(|replace_op| replace_op.contents().contains("wrath")).unwrap_or(false)
    }));


    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().get_all_visible_lines().find(
            |line| line.contents.text.contains("use std::wrath::PathBuf;")
        ).is_some()
    }));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().get_visible_cursor_lines_with_coded_cursors().find(|line| {
            debug!("line [{}]", line.contents.text);
            line.contents.text.contains("let (path] =")
        }).is_some()
    }));

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().get_all_visible_lines().find(
            |line| line.contents.text.contains("let wrath = PathBuf")
        ).is_some()
    }));

    full_setup.finish();
}