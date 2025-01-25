use crate::big_tests::test_utils::open_context_and_select;
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

        // error!("{:?}", line);
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
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_basic_test_1")
        .with_files(["src/sometext.txt"])
        .build();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup.send_input(Keycode::End.to_key().to_input_event());

    assert!(full_setup.wait_for(|f| {
        let line = f
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .next()
            .unwrap();

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

fn assert_main_rs(full_setup: &mut FullSetup) {
    assert!(full_setup.wait_for(|f| {
        let line = f.get_first_editor().unwrap().get_visible_cursor_lines().next().unwrap();

        line.contents.text.contains("use std::path::PathBuf;")
    }));
}

fn assert_sometext_txt(full_setup: &mut FullSetup) {
    assert!(full_setup.wait_for(|f| {
        let line = f.get_first_editor().unwrap().get_visible_cursor_lines().next().unwrap();

        line.contents.text.contains("here is some indented text")
    }));
}

#[test]
fn prev_next_works() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_basic_test_1")
        .with_files(["src/sometext.txt", "src/main.rs"])
        .build();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert_main_rs(&mut full_setup);

    full_setup.send_input(full_setup.config().keyboard_config.global.next_display.to_input_event());

    assert_sometext_txt(&mut full_setup);

    full_setup.send_input(full_setup.config().keyboard_config.global.next_display.to_input_event());

    assert_main_rs(&mut full_setup);

    full_setup.send_input(full_setup.config().keyboard_config.global.prev_display.to_input_event());

    assert_sometext_txt(&mut full_setup);

    full_setup.send_input(full_setup.config().keyboard_config.global.prev_display.to_input_event());

    assert_main_rs(&mut full_setup);
}

#[test]
fn prev_next_from_everything_bar_works() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/main_basic_test_1")
        .with_files(["src/sometext.txt", "src/main.rs"])
        .build();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert_main_rs(&mut full_setup);

    open_context_and_select(&mut full_setup, "next display");

    assert_sometext_txt(&mut full_setup);

    open_context_and_select(&mut full_setup, "next display");

    assert_main_rs(&mut full_setup);

    open_context_and_select(&mut full_setup, "previous display");

    assert_sometext_txt(&mut full_setup);

    open_context_and_select(&mut full_setup, "previous display");

    assert_main_rs(&mut full_setup);
}
