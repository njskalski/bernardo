use log::debug;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::treeview_interpreter::TreeViewInterpreterItem;
use crate::spath;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/fuzzy_file_open_test_1")
        .build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));

    full_setup
}

#[test]
fn fuzzy_file_opens() {
    let mut full_setup = common_start();

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.fuzzy_file));

    assert!(full_setup.wait_for(|f| f.get_fuzzy_search().is_some()));

    full_setup.finish();
}