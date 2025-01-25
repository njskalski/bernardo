use crate::config::config::Config;
use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut config = Config::default();
    config.global.tabs_to_spaces = None;

    let mut full_setup: FullSetup = FullSetup::new("./test_envs/tab_test_1")
        .with_config(config)
        .with_files(["file_with_tabs.txt"].iter())
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup
}

#[test]
fn tab_test_1() {
    let mut f = common_start();

    let lines: Vec<_> = f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines_with_coded_cursors()
        .collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].contents.text.as_str(), "#\ttab⏎")
}

#[test]
fn tab_test_2() {
    let mut f = common_start();

    let lines: Vec<_> = f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines_with_coded_cursors()
        .collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].contents.text.as_str(), "#\ttab⏎");

    assert!(f.send_key(Keycode::End.to_key().with_shift()));

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|i| i.contents.text)
            .find(|i| i == "(\ttab]⏎")
            .is_some()
    }));

    assert!(f.send_key(Keycode::Backspace.to_key()));

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|i| i.contents.text)
            .find(|i| {
                println!("{}", i);
                i == "#⏎"
            })
            .is_some()
    }));
}

#[test]
fn tab_test_3() {
    let mut f = common_start();

    assert!(f.send_key(Keycode::End.to_key()));
    assert!(f.send_key(Keycode::Enter.to_key()));

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 2)
            .is_some()
    }));

    let x: Vec<_> = f
        .get_first_editor()
        .unwrap()
        .get_all_visible_lines()
        .filter(|i| !i.contents.text.is_empty())
        .collect();

    f.send_key(Keycode::Tab.to_key());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str() == "\t#⏎")
            .is_some()
    }));

    f.send_key(Keycode::Tab.to_key());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str() == "\t\t#⏎")
            .is_some()
    }));

    f.send_key(Keycode::Tab.to_key());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str() == "\t\t\t#⏎")
            .is_some()
    }));

    f.send_key(Keycode::ArrowLeft.to_key());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str() == "\t\t#\t⏎")
            .is_some()
    }));

    f.send_key(Keycode::ArrowLeft.to_key());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str() == "\t#\t\t⏎")
            .is_some()
    }));

    f.send_key(Keycode::Tab.to_key());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str() == "\t\t#\t\t⏎")
            .is_some()
    }));
}

#[test]
fn tab_test_4() {
    let mut f = common_start();

    f.send_key(Keycode::Char('n').to_key().with_ctrl());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|i| i.visible_idx == 1 && i.contents.text.contains("⇱"))
            .is_some()
    }));

    f.send_key(Keycode::Tab.to_key());

    f.wait_frame();
    let x: Vec<_> = f
        .get_first_editor()
        .unwrap()
        .get_all_visible_lines_raw()
        .filter(|i| !i.contents.text.is_empty())
        .collect();

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_all_visible_lines_raw()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str().trim() == "|--|⏎")
            .is_some()
    }));

    f.send_key(Keycode::Tab.to_key());

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_all_visible_lines_raw()
            .find(|i| i.visible_idx == 2 && i.contents.text.as_str().trim() == "|--||--|⏎")
            .is_some()
    }));
}
