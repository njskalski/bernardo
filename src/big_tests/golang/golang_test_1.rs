use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use std::time::Duration;

fn get_full_setup(file: &str) -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/golang_1")
        .with_files([file])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn golang_is_highlighted() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("main.go");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    let vec: Vec<_> = full_setup.get_first_editor().unwrap().consistent_items_iter().collect();

    assert_eq!(
        vec.iter()
            .find(|item| item.text.contains("main"))
            .expect("no main found")
            .text_style
            .foreground,
        full_setup.get_theme().name_to_color("identifier").unwrap()
    );

    assert_eq!(
        vec.iter()
            .find(|item| item.text.contains("import"))
            .expect("no import found")
            .text_style
            .foreground,
        full_setup.get_theme().name_to_color("keyword").unwrap()
    );

    assert_eq!(
        vec.iter()
            .find(|item| item.text.contains("func"))
            .expect("no func found")
            .text_style
            .foreground,
        full_setup.get_theme().name_to_color("keyword").unwrap()
    );
}

#[test]
fn golang_lsp_shows_labels() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("main.go");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    for _ in 0..5 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
    }
    full_setup.send_key(Keycode::End.to_key());
    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|f| f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_line_indices()
        .find(|item| item.visible_idx == 7)
        .is_some()));

    full_setup.type_in("fmt.");
    // full_setup.send_key(full_setup.config().keyboard_config.editor.request_completions);

    assert!(full_setup.wait_for(|f| f
        .get_first_editor()
        .unwrap()
        .get_errors()
        .find(|item| item.contents.text.starts_with("expected selector"))
        .is_some()));
}
