use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

#[test]
fn dropping_cursor_test_1() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/dropping_cursor_test_1")
        .with_files(["file_to_edit.txt"])
        .build();

    assert!(full_setup.wait_for(|full_setup| full_setup.is_editor_opened()));
    assert_eq!(
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .map(|c| c.visible_idx)
            .collect::<Vec<usize>>(),
        vec![1]
    );

    full_setup.send_key(Keycode::End.to_key());

    assert!(full_setup.wait_for(|full_setup| -> bool {
        let lines: Vec<String> = full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|line| line.contents.text)
            .collect();

        lines == vec!["some line no1#⏎".to_string()]
    }));

    full_setup.send_key(Keycode::ArrowLeft.to_key().with_ctrl());

    assert!(full_setup.wait_for(|full_setup| -> bool {
        let lines: Vec<String> = full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|line| line.contents.text)
            .collect();

        lines == vec!["some line #no1⏎".to_string()]
    }));

    full_setup.send_key(full_setup.config().keyboard_config.editor.enter_cursor_drop_mode);

    for _ in 0..3 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
        full_setup.send_key(Keycode::Enter.to_key());
    }

    full_setup.send_key(Keycode::Esc.to_key());

    assert!(full_setup.wait_for(|full_setup| -> bool {
        let lines: Vec<String> = full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|line| line.contents.text)
            .collect();

        lines == vec!["some line #no1⏎", "some line #no2⏎", "some line #no3⏎", "some line #no4⇱"]
    }));

    full_setup.send_key(Keycode::Home.to_key().with_ctrl().with_shift());

    assert!(full_setup.wait_for(|full_setup| -> bool {
        let lines: Vec<String> = full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|line| line.contents.text)
            .collect();

        // TODO yeah, I am not sure if this is the semantics of next-prev I want, but for now I stick with
        // it, no time to fix it.
        lines == vec!["[some line )no1⏎", "[some line )no2⏎", "[some line )no3⏎", "[some line )no4⇱"]
    }));

    full_setup.type_in("ugabuga!");

    assert!(full_setup.wait_for(|full_setup| -> bool {
        let lines: Vec<String> = full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|line| line.contents.text)
            .collect();

        // TODO yeah, I am not sure if this is the semantics of next-prev I want, but for now I stick with
        // it, no time to fix it.
        lines == vec!["ugabuga!#no1⏎", "ugabuga!#no2⏎", "ugabuga!#no3⏎", "ugabuga!#no4⇱"]
    }));

    full_setup.finish();
}

#[test]
fn dropping_cursor_scroll_test_2() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/dropping_cursor_test_1")
        .with_files(["file_to_test_scroll.txt"])
        .build();

    assert!(full_setup.wait_for(|full_setup| full_setup.is_editor_opened()));
    assert_eq!(
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .map(|c| c.visible_idx)
            .collect::<Vec<usize>>(),
        vec![1]
    );

    full_setup.send_key(full_setup.config().keyboard_config.editor.enter_cursor_drop_mode);
    for _ in 0..3 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
        full_setup.send_key(Keycode::Enter.to_key());
    }

    for _ in 0..100 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
    }

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .filter(|idx| idx.visible_idx == 104)
            .next()
            .is_some()
    }));
}
