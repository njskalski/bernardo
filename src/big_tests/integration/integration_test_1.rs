use log::debug;

use std::process::Command;
use crate::mocks::full_setup::FullSetup;
use crate::io::keys::Keycode;

#[test]
fn integration_test_with_rust_analyzer() {
    let _ra_process = Command::new("rust-analyzer")
        .arg("some_arg")
        .spawn()
        .expect("Failed to start rust-analyzer process");

    let mut full_setup: FullSetup = FullSetup::new("./test_envs/integration_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()), "Editor not opened");

    for _ in 0..4 {
        let key_sent = full_setup.send_key(Keycode::ArrowDown.to_key());
        assert!(key_sent, "Failed to send key");
    }

    // Simulate user typing
    let typing_result = full_setup.type_in("path.");
    assert!(typing_result, "Failed to type in");
    
    // Make sure we have an editor, then proceed
    let first_editor = full_setup.get_first_editor()
        .expect("Failed to get first editor");

    // Use iterator for visible_cursor_lines
    let visible_cursor_lines: Vec<_> = first_editor.get_visible_cursor_lines()
        .collect();

    // Make sure we have a visible cursor line
    assert!(!visible_cursor_lines.is_empty(), "No visible cursor lines");

    full_setup.finish();
}

fn get_rust_analyzer_completions(text: &str) -> Vec<String> {
    debug!("Getting completions for {}", text);
    vec!["into_boxed_path".to_string()]
}