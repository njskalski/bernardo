use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

pub fn open_context_and_select(full_setup: &mut FullSetup, name_infix: &str) {
    if full_setup.get_first_context_menu().is_none() {
        assert!(full_setup.send_input(full_setup.config().keyboard_config.global.everything_bar.to_input_event()));

        assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_context_menu().is_some() }));
    }

    assert!(full_setup.type_in(name_infix));

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_context_menu()
            .unwrap()
            .selected_option()
            .unwrap()
            .contains(name_infix)
    }));

    assert!(full_setup.send_input(Keycode::Enter.to_key().to_input_event()));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_context_menu().is_none() }));
}
