use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use std::time::Duration;

fn get_full_setup(file: &str) -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/python_test_1")
        .with_files([file])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    full_setup.send_key(Keycode::PageDown.to_key());
    assert!(full_setup.wait_for(|f| f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_line_indices()
        .find(|idx| idx.visible_idx == 13)
        .is_some()));

    full_setup
}

#[test]
fn python_is_highlighted() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("hello_world.py");

    let vec: Vec<_> = full_setup.get_first_editor().unwrap().consistent_items_iter().collect();

    assert_eq!(
        vec.iter().find(|item| item.text.contains("class")).unwrap().text_style.foreground,
        full_setup.get_theme().name_to_color("keyword").unwrap()
    );

    assert_eq!(
        vec.iter().find(|item| item.text.contains("def")).unwrap().text_style.foreground,
        full_setup.get_theme().name_to_color("keyword").unwrap()
    );

    assert_eq!(
        vec.iter().find(|item| item.text.contains("Greeter")).unwrap().text_style.foreground,
        full_setup.get_theme().name_to_color("identifier").unwrap()
    );
}

#[test]
fn python_lsp_shows_completions() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("hello_world.py");

    full_setup.send_key(full_setup.config().keyboard_config.editor.request_completions);

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().completions().is_some()));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().completions().unwrap().is_loading() == false));

    let completions: Vec<_> = full_setup
        .get_first_editor()
        .unwrap()
        .completions()
        .unwrap()
        .items()
        .map(|item| item.text.trim().to_string())
        .collect();

    assert!(completions.len() > 3);

    let expected: Vec<_> = vec!["abs(x)", "assert", "await"];

    for e in expected {
        assert!(completions.contains(&e.to_string()));
    }
}
