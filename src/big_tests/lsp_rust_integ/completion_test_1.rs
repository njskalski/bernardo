use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use log::error;
use std::thread;
use std::time::Duration;

fn get_full_setup() -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/lsp_rust_integ_2")
        .with_files(["src/main.rs"])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn rust_lsp_completion_test_1() {
    if std::env::var("CI").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // TODO this should be replaced with "waiting for LSP to be ready", when some kind of statusbar
    // is implemented to signal presence of NavComp
    thread::sleep(Duration::from_secs(2));
    // full_setup.send_input(InputEvent::Tick);

    assert_eq!(
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .map(|c| c.visible_idx)
            .collect::<Vec<usize>>(),
        vec![1]
    );

    //pub fn some_function(x: &str) {
    for _ in 0..53 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .find(|c| c.visible_idx == 54)
            .is_some()
    }));

    assert!(full_setup.send_key(Keycode::End.to_key()));

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .find(|item| item.contents.text.contains("path.#"))
            .is_some()
    }));

    full_setup.send_key(full_setup.config().keyboard_config.editor.request_completions);

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_editor().unwrap().completions().is_some() }));

    // waiting for "loading" to disappear
    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .completions()
            .unwrap()
            .items()
            .find(|item| item.text.contains("loading..."))
            .is_none()
    }));

    let all_items: Vec<String> = full_setup
        .get_first_editor()
        .unwrap()
        .completions()
        .unwrap()
        .items()
        .map(|item| item.text)
        .collect();

    let some_items: Vec<_> = vec!["into_os_string", "into_boxed_path", "clamp", "capacity", "partial_cmp"];

    for sitem in some_items {
        assert!(
            all_items.iter().find(|item| item.contains(sitem)).is_some(),
            "couldn't find {} among [{:?}]",
            sitem,
            all_items
        );
    }
}
