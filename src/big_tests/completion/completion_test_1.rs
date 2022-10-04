use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::mocks::full_setup::{FullSetup, FullSetupBuilder};
use crate::mocks::mock_navcomp_provider::MockCompletionMatcher;
use crate::spath;
use crate::w7e::navcomp_provider::Completion;
use crate::w7e::navcomp_provider::CompletionAction::Insert;

#[test]
fn completion_test_1() {
    let mut full_setup: FullSetup = FullSetupBuilder::new("./test_envs/completion_test_1")
        .with_files(["src/main.rs"])
        .build();

    let file = spath!(full_setup.fsf(), "src", "main.rs").unwrap();

    assert!(full_setup.wait_frame());
    assert!(full_setup.is_editor_opened());
    assert!(full_setup.navcomp_pilot().wait_for_load(&file).is_some());


    assert_eq!(full_setup.get_first_editor().unwrap().get_visible_cursor_line_indices().map(|c| c.visible_idx).collect::<Vec<usize>>(), vec![1]);

    for _ in 0..4 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().get_visible_cursor_line_indices().map(|c| c.visible_idx).next() == Some(5)));

    assert!(full_setup.type_in("path."));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().get_visible_cursor_lines().next().unwrap().contents.contains("path.")));

    full_setup.navcomp_pilot().completions().unwrap().push(
        MockCompletionMatcher {
            path: None,
            answer: Some(vec![
                Completion {
                    key: "into_os_string".to_string(),
                    desc: None,
                    action: Insert("into_os_string".to_string()),
                },
                Completion {
                    key: "into_boxed_path".to_string(),
                    desc: None,
                    action: Insert("into_boxed_path".to_string()),
                },
            ]),
        }
    );

    assert!(full_setup.send_key(Keycode::Space.to_key().with_ctrl()));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().completions().map(|comp| comp.highlighted(true).is_some()).unwrap_or(false)
    }));

    assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));

    full_setup.screenshot();

    assert!(full_setup.wait_for(|full_setup| {
        full_setup.get_first_editor().unwrap().completions().map(|comp| comp.highlighted(true).unwrap().1
            == "into_boxed_path").unwrap_or(false)
    }));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().get_visible_cursor_lines().next().unwrap().contents.contains("path.into_boxed_path")));

    let end = full_setup.finish();
    assert!(end.screenshot());
}