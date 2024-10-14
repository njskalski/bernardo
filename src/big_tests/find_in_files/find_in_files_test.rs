use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/find_in_files_test_1")
        // .with_frame_based_wait()
        .build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));

    full_setup
}

#[test]
fn find_in_files_opens() {
    let mut full_setup = common_start();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.find_in_files));
    assert!(full_setup.wait_for(|f| f.get_find_in_files().is_some()));
    assert!(full_setup.get_find_in_files().unwrap().is_focused());
}

