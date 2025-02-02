use crate::big_tests::test_utils::all_items_of_named_color;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use std::time::Duration;

fn get_full_setup(file: &str) -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/javascript_test_1")
        .with_files([file])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn javascript_is_highlighted() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("hello_world.js");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert!(all_items_of_named_color(
        &mut full_setup,
        "keyword",
        vec!["function", "return", "const"]
    ));
    assert!(all_items_of_named_color(
        &mut full_setup,
        "variable",
        vec!["name", "message", "console"]
    ));
    assert!(all_items_of_named_color(
        &mut full_setup,
        "function",
        vec!["getGreeting", "main", "getGreeting"]
    ));
    assert!(all_items_of_named_color(
        &mut full_setup,
        "comment",
        vec!["function", "to", "return", "a", "greeting", "message"]
    ));

    // against false postitives
    assert!(full_setup.get_theme().tm.name_to_color("comment") != full_setup.get_theme().tm.name_to_color("function"));
}
