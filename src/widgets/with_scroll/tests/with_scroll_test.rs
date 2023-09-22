use std::rc::Rc;

use crate::experiments::screen_shot::screenshot;
use crate::widgets::with_scroll::tests::with_scroll_view_testbed::WithScrollTestbed;

fn get_setup() -> WithScrollTestbed {
    let mut testbed = WithScrollTestbed::new();
    {
        let mut list = testbed.widget.internal_mut();

        let items: Vec<Rc<String>> = (0..50).map(|idx| Rc::new(format!("item{}", idx))).collect();

        list.set_provider(Box::new(items));
    }

    testbed
}

#[test]
fn basic_with_scroll_testbed_test() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().is_some());

    {
        // let interpreter = setup.interpreter()?;
    }
    
    screenshot(&setup.frame_op().unwrap().buffer);
}

