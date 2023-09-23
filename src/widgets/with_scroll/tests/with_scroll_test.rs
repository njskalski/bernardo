use std::rc::Rc;

use test_log::test;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::widgets::with_scroll::tests::with_scroll_view_testbed::WithScrollTestbed;

fn get_setup() -> WithScrollTestbed {
    let mut testbed = WithScrollTestbed::new();
    {
        let mut list = testbed.widget.internal_mut();

        let items: Vec<Rc<String>> = (1..51).map(|idx| Rc::new(format!("item{}", idx))).collect();

        list.set_provider(Box::new(items));

        list.set_highlighted(0);
    }

    testbed
}

#[test]
fn basic_with_scroll_testbed_test_page_down_and_page_up_works() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().is_some());

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "20item19");
    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "21item20");

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "41item40");

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "50item49");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "31item30");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "30item29");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "10item9");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "1name");
}

