use crate::io::keys::Keycode;
use crate::mocks::mock_tree_item::get_mock_data_set_1;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::widgets::context_menu::tests::context_menu_testbed::ContextMenuTestbed;

pub fn get_setup_1() -> ContextMenuTestbed {
    let nested_menu_testbed = ContextMenuTestbed::new(get_mock_data_set_1());

    nested_menu_testbed
}

#[test]
fn context_menu_1_enter_expands() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(testbed.context_menu().unwrap().tree_view().items().len(), 1);
    assert!(testbed.has_items(["menu1"].into_iter()));

    testbed.push_input(Keycode::Enter.to_key().to_input_event());

    assert!(testbed.wait_for(|testbed| { testbed.has_items(["submenu"].into_iter()) }));
}

#[test]
fn context_menu_2_letters_filter() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(testbed.context_menu().unwrap().tree_view().items().len(), 1);
    assert_eq!(
        testbed.context_menu().unwrap().tree_view().items().iter().next().unwrap().label,
        "menu1"
    );

    testbed.push_input(Keycode::Enter.to_key().to_input_event());
    testbed.push_text("oo");

    assert_eq!(testbed.context_menu().unwrap().editbox().contents(), "oo".to_string());

    testbed.screenshot();
}
