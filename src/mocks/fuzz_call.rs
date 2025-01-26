use crate::config::config::Config;
use crate::io::keys::{Key, Keycode};
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut config = Config::default();
    config.global.tabs_to_spaces = None;

    let mut full_setup: FullSetup = FullSetup::new("./test_envs/fuzz_test_1").with_config(config).build();

    full_setup.send_key(Keycode::Char('n').to_key().with_ctrl());

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().is_editor_focused()));

    full_setup
}

pub fn fuzz_call(inputs: Vec<Key>) {
    let mut f = common_start();

    for key in inputs {
        f.send_key(key);
        f.wait_frame();
    }
}
