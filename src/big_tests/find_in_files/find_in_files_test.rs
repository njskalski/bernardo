use std::thread::sleep;
use std::time::Duration;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::mocks::editor_interpreter::LineIdxTuple;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/find_in_files_test_1")
        .with_timeout(Duration::from_secs(5))
        // .with_frame_based_wait()
        .build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));

    full_setup
}

#[test]
fn find_in_files_opens() {
    let mut f = common_start();

    assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
    assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
    assert!(f.get_find_in_files().unwrap().is_focused());

    f.type_in("min");

    assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().query_box().contents().contains("min") }));

    f.send_input(Keycode::Enter.to_key().to_input_event());

    assert!(f.wait_for(|f| { f.get_code_results_view().is_some() }));

    // TODO remove tick
    sleep(Duration::from_secs(1));
    f.send_input(InputEvent::Tick);

    // displays
    assert!(f.wait_for(|f| { f.get_code_results_view().unwrap().editors().len() == 3 }));

    f.screenshot();

    assert!(f.wait_for(|f| {
        f.get_code_results_view()
            .unwrap()
            .editors()
            .first()
            .map(|editor| {
                editor
                    .get_visible_cursor_lines()
                    .find(|line| line.contents.text.starts_with("distinctio. Nam libero tempore"))
                    .is_some()
            })
            .unwrap_or(false)
    }));

    f.send_input(Keycode::Enter.to_key().to_input_event());

    assert!(f.wait_for(|f| f.get_code_results_view().is_none()));
    assert!(f.wait_for(|f| { f.get_first_editor().is_some() }));

    let lines: Vec<LineIdxTuple> = f.get_first_editor().unwrap().get_all_visible_lines().collect();

    assert!(lines[0]
        .contents
        .text
        .starts_with("\"At vero eos et accusamus et iusto odio dignissimos"));
    assert!(lines[4]
        .contents
        .text
        .starts_with("placeat facere possimus, omnis voluptas assumenda"));
}

#[test]
fn find_in_files_hit_on_empty_res() {
    let result = std::panic::catch_unwind(|| {
        let mut f = common_start();

        assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
        assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
        assert!(f.get_find_in_files().unwrap().is_focused());

        f.type_in("rzeszów");
        f.send_input(Keycode::Enter.to_key().to_input_event());
        assert!(f.wait_for(|f| f.get_code_results_view().is_some()));
        f.screenshot();
        //Below input triggers panic, but in different thread and it's not properly propagated
        //Issue is that big list doesnt handle hit on empty
        f.send_input(Keycode::Enter.to_key().to_input_event());
        assert!(f.wait_for(|f| f.get_code_results_view().is_some()));

        //Below code only ensures that mentioned above thread didn't panic.
        assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
        assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
        assert!(f.get_find_in_files().unwrap().is_focused());

        f.type_in("rzeszów");
        f.send_input(Keycode::Enter.to_key().to_input_event());
        assert!(f.wait_for(|f| f.get_code_results_view().is_some()));
    });

    assert!(result.is_ok(), "Test panicked");
}

#[test]
fn find_in_files_nonempty_filter() {
    let mut f = common_start();

    assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
    assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
    assert!(f.get_find_in_files().unwrap().is_focused());

    f.type_in("min");

    assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().query_box().contents().contains("min") }));

    f.send_input(Keycode::ArrowDown.to_key().to_input_event());

    assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().filter_box().is_focused() }));

    f.type_in("*.txt");

    assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().filter_box().contents().contains("*.txt") }));

    f.send_input(Keycode::Enter.to_key().to_input_event());

    assert!(f.wait_for(|f| { f.get_find_in_files().is_none() }));
    assert!(f.wait_for(|f| { f.get_code_results_view().is_some() }));

    // 3, not 2.
    assert!(f.wait_for(|f| { f.get_code_results_view().unwrap().editors().len() == 2 }));
}
