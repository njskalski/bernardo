use crate::experiments::screen_shot::screenshot;
use crate::widgets::nested_menu::tests::nested_menu_testbed::NestedMenuTestbed;

pub fn get_setup() -> NestedMenuTestbed {
    let nested_menu_testbed = NestedMenuTestbed::new();

    nested_menu_testbed
}

#[test]
fn nested_menu_1() {
    let mut testbed = NestedMenuTestbed::new();

    testbed.next_frame();
    assert_eq!(testbed.frame_op().unwrap().get_nested_menus().count(), 1);

    assert_eq!(testbed.nested_menu().unwrap().get_selected_item().label, "option1".to_string());

    screenshot(&testbed.frame_op().unwrap().buffer);
}
