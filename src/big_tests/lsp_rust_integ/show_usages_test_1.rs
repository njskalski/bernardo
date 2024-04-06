use std::time::Duration;

use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn get_full_setup() -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/lsp_rust_integ_1")
        .with_files(["src/some_other_file.rs"])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(10))
        .build();

    full_setup
}

#[test]
fn show_usages_integ_test_1() {
    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert_eq!(
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .map(|c| c.visible_idx)
            .collect::<Vec<usize>>(),
        vec![1]
    );

    assert!(full_setup
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .find(|line| line.visible_idx == 1)
        .is_some());

    //pub fn some_function(x: &str) {
    for _ in 0..7 {
        assert!(full_setup.send_key(Keycode::ArrowRight.to_key()));
    }

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_cells()
            .find(|(pos, cell)| cell.grapheme() == Some("s"))
            .is_some()
    }));

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines()
            .find(|line| line.contents.text.contains("some_function(x"))
            .is_some()
    }));

    full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar);

    assert!(full_setup.wait_for(|f| { f.get_first_editor().unwrap().context_bar_op().is_some() }));

    assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .context_bar_op()
            .map(|c| c.selected_option().map(|c| c.trim().starts_with("show usages")).unwrap_or(false))
            .unwrap_or(false)
    }));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_code_results_view().is_some() }));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_code_results_view().unwrap().editors().len() == 4 }));

    {
        full_setup.screenshot();
        let results_view = full_setup.get_code_results_view().unwrap();
        let editors = results_view.editors();

        assert_eq!(editors[0].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(3));
        assert_eq!(editors[1].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(8));
        assert_eq!(editors[2].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(12));
        assert_eq!(editors[3].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(1));
    }
}
