use log::debug;

use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::mock_navcomp_provider::MockCompletionMatcher;
use crate::primitives::printable::Printable;
use crate::w7e::navcomp_provider::Completion;
use crate::w7e::navcomp_provider::CompletionAction::Insert;

#[test]
fn show_usages_integ_test_1() {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/show_usages_test_1")
        .with_files(["src/main.rs"])
        // .with_frame_based_wait()
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert_eq!(full_setup.get_first_editor().unwrap().get_visible_cursor_line_indices().map(|c| c.visible_idx).collect::<Vec<usize>>(), vec![1]);

    for _ in 0..7 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().get_visible_cursor_lines().find(|line| line.visible_idx == 8).is_some()));

    assert!(full_setup.send_key(Keycode::ArrowRight.to_key().with_ctrl()));

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor().unwrap().get_visible_cursor_lines().find(|line| line.contents.text.trim() == "some_function(\"a\");⏎").is_some()
    }));

    full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar);

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor().unwrap().context_bar_op().is_some()
    }));

    assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    assert!(full_setup.wait_for(|f| {
        f.get_first_editor().unwrap().context_bar_op().map(|c| c.selected_option().map(|c| c.trim().starts_with("show usages")).unwrap_or(false)).unwrap_or(false)
    }));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));


    full_setup.wait_frame();
    full_setup.screenshot();
}