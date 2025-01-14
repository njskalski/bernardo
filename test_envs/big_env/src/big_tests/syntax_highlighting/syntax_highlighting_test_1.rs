use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

// This test is not super advanced, but I have bigger fish to fry than implementing yet another
// parsed output iterator

#[test]
fn syntax_highlighting_test_1_not_all_of_one_color() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/syntax_highlighting_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert_eq!(
        false,
        full_setup
            .get_first_editor()
            .unwrap()
            .get_all_visible_lines()
            .map(|line| -> bool {
                line.contents.text_style.is_some() // true iff style was uniform over the line
            })
            .fold::<bool, fn(bool, bool) -> bool>(true, |all_uniform, current_uniform| -> bool { all_uniform && current_uniform })
    );
}
