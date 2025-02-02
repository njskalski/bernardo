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

    let keyword_color = full_setup.get_theme().tm.name_to_color("keyword").unwrap();
    let variable_color = full_setup.get_theme().tm.name_to_color("variable").unwrap();
    let function_call_color = full_setup.get_theme().tm.name_to_color("function_call").unwrap();

    let items: Vec<_> = full_setup.get_first_editor().unwrap().consistent_items_iter().collect();

    let function_calls = items
        .iter()
        .filter(|item| item.text_style.foreground == function_call_color)
        .map(|item| item.text.clone())
        .collect::<Vec<_>>();

    for e in vec!["Main", "System.Environment", "greet", "main", "head", "putStrLn"] {
        assert!(function_calls.iter().any(|item| item.contains(e)), "did not find function_call {}", e);
    }

    let variables = items
        .iter()
        .filter(|item| item.text_style.foreground == variable_color)
        .map(|item| item.text.clone())
        .collect::<Vec<_>>();

    for e in vec!["name", "args"] {
        assert!(variables.iter().any(|item| item.contains(e)), "did not find variable {}", e);
    }

    let keywords = items
        .iter()
        .filter(|item| item.text_style.foreground == keyword_color)
        .map(|item| item.text.clone())
        .collect::<Vec<_>>();

    for e in vec!["module", "where", "import", "do", "let", "if", "then", "else"] {
        assert!(keywords.iter().any(|item| item.contains(e)), "did not find keywords {}", e);
    }
}
