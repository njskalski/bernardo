use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use std::time::Duration;

fn get_full_setup(file: &str) -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/bash_test_1")
        .with_files([file])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn bash_is_highlighted() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("example_script.sh");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    let function_color = full_setup.get_theme().tm.name_to_color("function").unwrap();
    let string_color = full_setup.get_theme().tm.name_to_color("string").unwrap();

    let items: Vec<_> = full_setup.get_first_editor().unwrap().consistent_items_iter().collect();

    let functions = items
        .iter()
        .filter(|item| item.text_style.foreground == function_color)
        .collect::<Vec<_>>();
    // let constants = items.iter().filter(|item| item.text_style.foreground == constant_color).collect::<Vec<_>>();
    let strings = items
        .iter()
        .filter(|item| item.text_style.foreground == string_color)
        .collect::<Vec<_>>();

    let expected_function_names = vec!["greet_user", "say_goodbye", "main"];
    let expected_strings = vec!["Enter", "your", "name:"];

    for e in expected_function_names {
        assert!(functions.iter().any(|item| item.text.contains(e)), "did not find function {}", e);
    }

    for e in expected_strings {
        assert!(strings.iter().any(|item| item.text.contains(e)), "did not find function {}", e);
    }
}
