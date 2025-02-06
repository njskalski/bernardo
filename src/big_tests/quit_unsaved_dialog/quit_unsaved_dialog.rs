use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/quit_unsaved_confirm_dialog_1").build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));

    assert!(full_setup.send_key(full_setup.config().keyboard_config.global.new_buffer));
    assert!(full_setup.wait_for(|f| f.get_first_editor().is_some()));
    assert!(full_setup.get_first_editor().unwrap().is_editor_focused());

    full_setup.type_in("sometext");

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines()
            .find(|item| item.contents.text.contains("sometext"))
            .is_some()
    }));

    full_setup.send_key(full_setup.config().keyboard_config.global.quit);

    assert!(full_setup.wait_for(|f| { f.get_first_generic_dialog().is_some() }));

    full_setup
}

#[test]
fn quit_unsaved_dialog_shows() {
    let mut f = common_start();

    assert!(f.get_first_generic_dialog().is_some());
    assert!(f
        .get_first_generic_dialog()
        .unwrap()
        .get_text()
        .contents()
        .contains("You have unsaved buffers."));
}

#[test]
fn arrows_work() {
    let mut f = common_start();

    assert!(f.get_first_generic_dialog().is_some());
    assert!(f
        .get_first_generic_dialog()
        .unwrap()
        .get_button_by_text("Go back")
        .unwrap()
        .is_focused());

    f.send_key(Keycode::ArrowRight.to_key());

    assert!(f.wait_for(|f| f
        .get_first_generic_dialog()
        .unwrap()
        .get_button_by_text("Discard and quit")
        .unwrap()
        .is_focused()));
}

#[test]
fn esc_closes() {
    let mut f = common_start();

    assert!(f.get_first_generic_dialog().is_some());

    f.send_key(Keycode::Esc.to_key());

    assert!(f.wait_for(|f| f.get_first_generic_dialog().is_none()));
}
