use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::spath;
use std::thread;
use std::time::Duration;

fn get_full_setup() -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/lsp_rust_integ_format_1")
        .with_files(["src/main.rs"])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

fn common_suffix(unformatted_file: &str, formatted_file: &str) {
    assert_ne!(formatted_file, unformatted_file);

    assert_eq!(
        formatted_file,
        r#"fn main() {
    let greeting = create_greeting("World");
    display_message(&greeting);
}

fn create_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn display_message(message: &str) {
    println!("{}", message);
}
"#
    );
}

#[test]
fn rust_lsp_format_test_with_shortcut() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // TODO this should be replaced with "waiting for LSP to be ready", when some kind of statusbar
    // is implemented to signal presence of NavComp
    thread::sleep(Duration::from_secs(2));
    // full_setup.send_input(InputEvent::Tick);

    let path = spath!(full_setup.fsf(), "src", "main.rs").unwrap();
    let unformatted_file = path.read_entire_file_to_string().unwrap();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.reformat));

    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.close_buffer));

    assert!(full_setup.wait_for(|full_setup| { full_setup.is_no_editor_opened() }));

    let formatted_file = path.read_entire_file_to_string().unwrap();

    common_suffix(&unformatted_file, &formatted_file);
}

#[test]
fn rust_lsp_format_test_with_everything_bar() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // TODO this should be replaced with "waiting for LSP to be ready", when some kind of statusbar
    // is implemented to signal presence of NavComp
    thread::sleep(Duration::from_secs(2));
    // full_setup.send_input(InputEvent::Tick);

    let path = spath!(full_setup.fsf(), "src", "main.rs").unwrap();
    let unformatted_file = path.read_entire_file_to_string().unwrap();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_context_menu().is_some() }));

    full_setup.type_in("format");

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_context_menu()
            .unwrap()
            .selected_option()
            .unwrap()
            .contains("format")
    }));

    full_setup.send_key(Keycode::Enter.to_key());
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save));
    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.close_buffer));

    assert!(full_setup.wait_for(|full_setup| { full_setup.is_no_editor_opened() }));

    let formatted_file = path.read_entire_file_to_string().unwrap();

    common_suffix(&unformatted_file, &formatted_file);
}
