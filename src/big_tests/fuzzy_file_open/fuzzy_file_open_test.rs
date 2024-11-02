use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/fuzzy_file_open_test_1")
        // .with_frame_based_wait()
        .build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));

    full_setup
}

#[test]
fn fuzzy_file_opens() {
    let mut full_setup = common_start();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.fuzzy_file));
    assert!(full_setup.wait_for(|f| f.get_fuzzy_search().is_some()));
    assert!(full_setup.get_fuzzy_search().unwrap().is_focused());

    full_setup.type_in("ain");

    assert!(full_setup.wait_for(|f| f.get_fuzzy_search().unwrap().editbox().contents().as_str() == "ain"));

    for _ in 0..3 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
    }

    assert!(full_setup.wait_for(|f| f
        .get_fuzzy_search()
        .unwrap()
        .selected_option()
        .unwrap()
        .as_str()
        .trim()
        .contains("main.rs")));

    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup.finish();
}

#[test]
fn fuzzy_search_esc_doesnt_crash() {
    let mut full_setup = common_start();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.fuzzy_file));
    assert!(full_setup.wait_for(|f| f.get_fuzzy_search().is_some()));

    assert!(full_setup.get_fuzzy_search().unwrap().is_focused());

    full_setup.send_key(Keycode::Esc.to_key());

    assert!(full_setup.wait_for(|f| f.get_fuzzy_search().is_none()));

    full_setup.finish();
}

#[test]
fn fuzzy_search_scroll_works() {
    let mut full_setup = common_start();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.fuzzy_file));
    assert!(full_setup.wait_for(|f| f.get_fuzzy_search().is_some()));

    assert!(full_setup.get_fuzzy_search().unwrap().is_focused());

    loop {
        let prev_highlighted = full_setup.get_fuzzy_search().unwrap().selected_option();
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));

        assert!(full_setup.wait_for(|full_setup| {
            let new_highlighted = full_setup.get_fuzzy_search().unwrap().selected_option();
            new_highlighted != prev_highlighted
        }));

        if full_setup
            .get_fuzzy_search()
            .unwrap()
            .selected_option()
            .map(|s| s.trim().contains("data45.txt"))
            .unwrap_or(false)
        {
            break;
        }
    }
}
