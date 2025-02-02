use crate::big_tests::test_utils::all_items_of_named_color;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use std::time::Duration;

fn get_full_setup(file: &str) -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/haskell_test_1")
        .with_files([file])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn haskell_is_highlighted() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("main.hs");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert!(all_items_of_named_color(
        &mut full_setup,
        "keyword",
        vec!["module", "where", "import", "do", "let", "if", "then", "else"]
    ));
    assert!(all_items_of_named_color(&mut full_setup, "variable", vec!["name", "args"]));
    assert!(all_items_of_named_color(
        &mut full_setup,
        "function",
        vec!["Main", "greet", "main", "putStrLn"]
    ));
    assert!(all_items_of_named_color(
        &mut full_setup,
        "comment",
        vec!["Function", "to", "generate", "message"]
    ));

    // against false postitives
    assert!(full_setup.get_theme().tm.name_to_color("comment") != full_setup.get_theme().tm.name_to_color("function"));
}
