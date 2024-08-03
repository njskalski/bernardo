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

    assert!(full_setup.wait_for(|f| f.get_fuzzy_search().unwrap().get_edit_box().contents().as_str() == "ain"));

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
