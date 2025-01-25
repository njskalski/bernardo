use std::thread;
use std::time::Duration;

use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn get_full_setup() -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/lsp_rust_integ_1")
        .with_files(["src/main.rs"])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(20))
        .build();

    full_setup
}

#[test]
fn go_to_definition_test_1() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // TODO this should be replaced with "waiting for LSP to be ready", when some kind of statusbar
    // is implemented to signal presence of NavComp
    thread::sleep(Duration::from_secs(6));
    // full_setup.send_input(InputEvent::Tick);

    assert!(full_setup.wait_for(|full_setup| full_setup
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .find(|line| line.visible_idx == 1)
        .is_some()));

    //pub fn some_function(x: &str) {
    for _ in 0..7 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|full_setup| full_setup
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .find(|line| line.visible_idx == 8)
        .is_some()));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_lines()
            .find(|line| line.contents.text.contains("some_function(\"a"))
            .is_some()
    }));

    for _ in 0..5 {
        assert!(full_setup.send_key(Keycode::ArrowRight.to_key()));
    }

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_cells()
            .find(|(pos, cell)| cell.grapheme() == Some("o"))
            .is_some()
    }));

    full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar);

    assert!(full_setup.wait_for(|f| { f.get_first_editor().unwrap().context_bar_op().is_some() }));

    for _ in 0..2 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .context_bar_op()
            .map(|c| {
                c.selected_option()
                    .map(|c| c.trim().starts_with("go to definition"))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|f| { f.get_code_results_view().map(|crv| { crv.editors().len() > 0 }).unwrap_or(false) }));

    assert!(full_setup.get_code_results_view().unwrap().editors().len() == 1);

    assert!(full_setup.wait_for(|f| {
        f.get_code_results_view()
            .unwrap()
            .editors()
            .first()
            .unwrap()
            .get_visible_cursor_lines()
            .find(|line| line.contents.text.contains("pub fn some_function(x: &str) {"))
            .is_some()
    }));

    //Here we test whether scrool follows cursor.
    full_setup.send_input(Keycode::Enter.to_key().to_input_event());

    assert!(full_setup.wait_for(|f| f.get_code_results_view().is_none()));
    assert!(full_setup.wait_for(|f| { f.get_first_editor().is_some() }));
    full_setup.screenshot();
    assert!(full_setup.wait_for(|f| { f.get_first_editor().unwrap().get_visible_cursor_cells().next().is_some() }));
}
