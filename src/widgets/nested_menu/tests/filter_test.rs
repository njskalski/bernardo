use crate::widgets::nested_menu::tests::mock_provider::get_mock_data_set_2;
use crate::widgets::nested_menu::tests::nested_menu_testbed::NestedMenuTestbed;

pub fn get_setup_2() -> NestedMenuTestbed {
    let nested_menu_testbed = NestedMenuTestbed::new(get_mock_data_set_2());

    nested_menu_testbed
}

#[test]
fn nested_menu_filter_1() {
    let mut testbed = get_setup_2();

    testbed.next_frame();
    assert_eq!(testbed.frame_op().unwrap().get_nested_menus().count(), 1);

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );
}
