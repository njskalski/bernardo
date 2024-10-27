use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/find_in_files_test_1")
        // .with_frame_based_wait()
        .build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));

    full_setup
}

#[ignore]
#[test]
fn find_in_files_opens() {
    let mut f = common_start();

    assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
    assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
    assert!(f.get_find_in_files().unwrap().is_focused());

    f.type_in("min");

    assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().query_box().contents().contains("min") }));

    f.send_input(Keycode::Enter.to_key().to_input_event());

    // assert!(f.wait_for(|f| { f.get_code_results_view().is_some() }));
}
