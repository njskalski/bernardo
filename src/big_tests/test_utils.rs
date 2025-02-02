use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use log::error;

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

pub fn all_items_of_named_color(full_setup: &FullSetup, color_name: &str, required_items: Vec<&str>) -> bool {
    let color = full_setup.get_theme().tm.name_to_color(color_name).unwrap();

    let found_items: Vec<_> = full_setup
        .get_first_editor()
        .unwrap()
        .consistent_items_iter()
        .filter(|item| item.text_style.foreground == color)
        .map(|item| item.text.clone())
        .collect();

    for e in required_items {
        if !found_items.iter().any(|item| item.contains(e)) {
            error!(
                "did not find expected item '{}' of color '{}' among {:?}",
                e, color_name, found_items
            );
            return false;
        }
    }
    true
}
