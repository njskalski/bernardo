use crate::experiments::screen_shot::screenshot;
use crate::widgets::editor_widget::tests::setup::get_setup;

#[test]
fn basic_editor_testbed_test() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().unwrap().is_editor_focused());

    {
        // let interpreter = setup.interpreter()?;
    }


    // screenshot(i);
}