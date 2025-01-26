use crate::config::config::Config;
use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut config = Config::default();
    config.global.tabs_to_spaces = None;

    let mut full_setup: FullSetup = FullSetup::new("./test_envs/tab_test_1").with_config(config).build();

    full_setup.send_key(Keycode::Char('n').to_key());

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup
}
