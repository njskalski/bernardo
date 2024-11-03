use std::rc::Rc;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widgets::with_scroll::tests::with_scroll_view_testbed::WithScrollTestbed;

fn get_setup() -> WithScrollTestbed {
    // logger_setup(true, None, None);

    let mut testbed = WithScrollTestbed::new();
    {
        let list = testbed.widget.internal_mut();

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

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item1");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "1 name");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(1).unwrap().trim(), "2 item1");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "20 item19");

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item21");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "3 item2");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "22 item21");

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item41");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "23 item22");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "42 item41");

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item50");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "32 item31");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "51 item50");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item30");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "31 item30");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "50 item49");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item10");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "11 item10");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "30 item29");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item1");
    // this is a special case, if highlighted is the first row, kite points to "name" row
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "1 name");
}

#[test]
fn with_scroll_visible_rect_offset() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().is_some());

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "1 name");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(1).unwrap().trim(), "2 item1");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "20 item19");
    assert_eq!(setup.widget.internal().get_highlighted().unwrap().as_str(), "item1");
    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item1");
    assert_eq!(setup.widget.scroll().offset, XY::ZERO);
    assert_eq!(setup.last_child_visible_rect().unwrap(), Rect::new(XY::ZERO, XY::new(6, 20)));

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "3 item2");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "22 item21");
    assert_eq!(setup.widget.internal().get_highlighted().unwrap().as_str(), "item21");
    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item21");
    assert_eq!(setup.widget.scroll().offset, XY::new(0, 2));
    assert_eq!(setup.last_child_visible_rect().unwrap(), Rect::new(XY::new(0, 2), XY::new(6, 20)));

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "23 item22");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "42 item41");
    assert_eq!(setup.widget.internal().get_highlighted().unwrap().as_str(), "item41");
    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item41");
    assert_eq!(setup.widget.scroll().offset, XY::new(0, 22));
    assert_eq!(setup.last_child_visible_rect().unwrap(), Rect::new(XY::new(0, 22), XY::new(6, 20)));

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "22 item21");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "41 item40");
    assert_eq!(setup.widget.internal().get_highlighted().unwrap().as_str(), "item21");
    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item21");
    assert_eq!(setup.widget.scroll().offset, XY::new(0, 21));
    assert_eq!(setup.last_child_visible_rect().unwrap(), Rect::new(XY::new(0, 21), XY::new(6, 20)));
}
