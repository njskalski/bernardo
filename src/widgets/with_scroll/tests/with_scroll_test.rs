use std::fmt::Debug;
use std::rc::Rc;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widgets::with_scroll::tests::with_scroll_view_testbed::WithScrollTestbed;

fn get_setup() -> WithScrollTestbed {
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

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "20 item19");
    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "21 item20");

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "41 item40");

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "50 item49");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "31 item30");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "30 item29");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "10 item9");

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));
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

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "2 item1");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "21 item20");
    assert_eq!(setup.widget.internal().get_highlighted().unwrap().as_str(), "item21");
    setup.screenshot();
    // TODO HERE so this test is failing, it seems like kite is followed properly, but over output/suboutput draws wrong lines

    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item21");
    assert_eq!(setup.widget.scroll().offset, XY::new(0, 1));
    assert_eq!(setup.last_child_visible_rect().unwrap(), Rect::new(XY::new(0, 1), XY::new(6, 20)));

    setup.send_input(InputEvent::KeyInput(Keycode::PageDown.to_key()));

    assert_eq!(setup.frame_op().unwrap().buffer.get_line(0).unwrap().trim(), "22 item21");
    assert_eq!(setup.frame_op().unwrap().buffer.get_line(19).unwrap().trim(), "41 item40");
    assert_eq!(setup.widget.internal().get_highlighted().unwrap().as_str(), "item41");
    assert_eq!(setup.observed_highlighted_op().unwrap().as_str(), "item41");
    assert_eq!(setup.widget.scroll().offset, XY::new(0, 21));
    assert_eq!(setup.last_child_visible_rect().unwrap(), Rect::new(XY::new(0, 21), XY::new(6, 20)));

    setup.send_input(InputEvent::KeyInput(Keycode::PageUp.to_key()));



    // let visible_rect = setup.widget.internal().get_last_size().unwrap().visible_rect();

    setup.screenshot();
    // assert_eq!(visible_rect.pos, XY::new(0, 20));


    // assert_eq!()
}
