use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::w7e::navcomp_provider::SymbolType::Key;
use log::error;

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

#[test]
fn rs_files_autoindent() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_basic_test_1").with_files(["src/main.rs"]).build();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    for _ in 0..7 {
        full_setup.send_input(Keycode::ArrowDown.to_key().to_input_event());
    }
    full_setup.send_input(Keycode::End.to_key().to_input_event());

    assert!(full_setup.wait_for(|f| {
        let line = f
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .next()
            .unwrap();

        line.visible_idx == 8 && line.contents.text.contains(";#")
    }));

    full_setup.send_input(Keycode::Enter.to_key().to_input_event());

    // after enter, line is prefixed with for spaces before the cursor
    assert!(full_setup.wait_for(|f| {
        let line = f
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .next()
            .unwrap();

        error!("{:?}", line);
        line.visible_idx == 9 && line.contents.text.starts_with("    #")
    }));

    full_setup.send_input(Keycode::Backspace.to_key().to_input_event());

    // after backspace it eats all whitespace
    assert!(full_setup.wait_for(|f| {
        let line = f
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .next()
            .unwrap();

        error!("{:?}", line);
        line.visible_idx == 9 && line.contents.text.starts_with("#")
    }));

    full_setup.send_input(Keycode::Backspace.to_key().to_input_event());

    // after second backspace it's back in line 8
    assert!(full_setup.wait_for(|f| {
        let line = f
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .next()
            .unwrap();

        line.visible_idx == 8 && line.contents.text.contains(");#")
    }));
}

#[test]
fn txt_file_no_autoindent() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_basic_test_1").with_files(["src/sometext.txt"]).build();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup.send_input(Keycode::End.to_key().to_input_event());

    assert!(full_setup.wait_for(|f| {
        let line = f
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .next()
            .unwrap();

        error!("line [{:?}]", line);
        line.visible_idx == 1 && line.contents.text.starts_with("    here is some indented text#")
    }));

    full_setup.send_input(Keycode::Enter.to_key().to_input_event());

    // after enter, line is NOT prefixed with any whitespace
    assert!(full_setup.wait_for(|f| {
        let line = f
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .next()
            .unwrap();

        line.visible_idx == 2 && line.contents.text.starts_with("#") // cursor immediately follows newline
    }));
}
