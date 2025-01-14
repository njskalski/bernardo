use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;

#[test]
fn context_kite_and_scroll_test_1() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/context_kite_and_scroll_1")
        .with_files(["src/main.rs"])
        .build();

    // let file = spath!(full_setup.fsf(), "src", "main.rs").unwrap();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert_eq!(
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .map(|c| c.visible_idx)
            .collect::<Vec<usize>>(),
        vec![1]
    );

    for _ in 0..50 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .next()
            .map(|cursor| cursor.visible_idx == 51)
            .unwrap_or(false)
    });

    assert!(full_setup.send_key(Keycode::ArrowRight.to_key().with_ctrl()));

    full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_cells()
            .next()
            .map(|item| item.1.grapheme() == Some("p"))
            .unwrap_or(false)
    });

    full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar);

    full_setup.wait_for(|full_setup| full_setup.get_first_context_menu().is_some());

    let rect = full_setup.get_first_context_menu().unwrap().meta().rect;
    assert!(Rect::from_zero(full_setup.get_frame().unwrap().buffer.size()).contains_rect(rect));

    let cursor_pos = full_setup
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .next()
        .unwrap()
        .contents
        .absolute_pos;
    assert!(!rect.contains(cursor_pos));
}

#[test]
fn context_kite_and_scroll_test_2() {
        let mut full_setup: FullSetup = FullSetup::new("./test_envs/context_kite_and_scroll_1")
            .with_files(["long_line.txt"])
            .build();

        assert!(full_setup.wait_for(|f| f.is_editor_opened()));
        assert_eq!(
            full_setup
                .get_first_editor()
                .unwrap()
                .get_visible_cursor_line_indices()
                .map(|c| c.visible_idx)
                .collect::<Vec<usize>>(),
            vec![1]
        );

        assert!(full_setup.send_key(Keycode::ArrowRight.to_key().with_ctrl())); //go to the end of long line
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key())); //go to end of short line
        full_setup.wait_for(|full_setup| {
            full_setup
                .get_first_editor()
                .unwrap()
                .get_visible_cursor_cells()
                .next()
                .map(|item| item.1.grapheme() == Some(""))
                .unwrap_or(false)
        });

        full_setup.screenshot();
        assert!(full_setup.get_first_editor().unwrap().get_visible_cursor_cells().next().is_some());
}
