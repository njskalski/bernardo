use std::thread;
use std::time::Duration;

use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn get_full_setup() -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/lsp_rust_integ_2")
        .with_files(["src/some_other_file.rs"])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn show_usages_integ_test_2() {
    if std::env::var("CI").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // TODO this should be replaced with "waiting for LSP to be ready", when some kind of statusbar
    // is implemented to signal presence of NavComp
    thread::sleep(Duration::from_secs(2));
    // full_setup.send_input(InputEvent::Tick);

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

    for _ in 0..3 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .context_bar_op()
            .map(|c| c.selected_option().map(|c| c.trim().starts_with("show usages")).unwrap_or(false))
            .unwrap_or(false)
    }));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_code_results_view().is_some() }));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_code_results_view().unwrap().editors().len() == 7 }));

    {
        // full_setup.screenshot();
        let results_view = full_setup.get_code_results_view().unwrap();
        let editors = results_view.editors();

        assert_eq!(editors[0].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(3));
        assert_eq!(editors[1].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(8));
        assert_eq!(editors[2].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(9));
        assert_eq!(editors[3].get_visible_cursor_lines().map(|line| line.visible_idx).next(), Some(10));
        //...
    }

    // we pick a THIRD use

    assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_code_results_view().unwrap().editors()[2].is_view_focused() }));

    // we HIT ENTER (and expect we go open next EDITOR view)

    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_code_results_view().is_none() }));
    assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_editor().is_some() }));

    // I test whether the right line is marked
    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines()
            .map(|line| line.visible_idx)
            .collect::<Vec<_>>()
            == vec![9]
    }));
}
