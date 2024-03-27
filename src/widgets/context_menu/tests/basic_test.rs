use crate::widgets::context_menu::tests::context_menu_testbed::ContextMenuTestbed;
use crate::widgets::context_menu::tests::mock_provider::get_mock_data_set_1;

pub fn get_setup_1() -> ContextMenuTestbed {
    let nested_menu_testbed = ContextMenuTestbed::new(get_mock_data_set_1());

    nested_menu_testbed
}

#[test]
fn context_menu_1() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(testbed.context_menu().unwrap().tree_view().items().len(), 1);

    testbed.screenshot();

    // assert_eq!(testbed.frame_op().unwrap().get_nested_menus().count(), 1);

    // assert_eq!(
    //     testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
    //     "option1".to_string()
    // );

    // let items = testbed.nested_menu().unwrap().get_items().collect::<Vec<_>>();

    // assert_eq!(
    //     items.iter().map(|item| item.label.clone()).collect::<Vec<_>>(),
    //     vec!["option1".to_string(), "option2".to_string(), "submenu".to_string()]
    // );

    // assert_eq!(items.iter().map(|item| item.leaf).collect::<Vec<_>>(), vec![true, true, false]);
}
