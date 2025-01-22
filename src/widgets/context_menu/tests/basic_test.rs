use crate::io::keys::Keycode;
use crate::mocks::mock_tree_item::get_mock_data_set_1;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::widgets::context_menu::tests::context_menu_testbed::{AdditionalData, ContextMenuTestbed, ContextMenuTestbedBuilder};

pub fn get_setup_1() -> ContextMenuTestbed {
    let setup = ContextMenuTestbedBuilder::new(AdditionalData {
        root: get_mock_data_set_1(),
    })
    .build();

    assert_eq!(setup.widget.tree_view().is_root_expanded(), false);
    assert_eq!(setup.widget.tree_view().is_filter_set(), false);

    // assert_eq!(setup.context_menu().unwrap().tree_view().items().len(), 1);

    setup
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
    testbed.screenshot();

    assert_eq!(testbed.context_menu().unwrap().tree_view().items().len(), 1);
    assert!(testbed.has_items(["menu1"].into_iter()));

    testbed.push_input(Keycode::Enter.to_key().to_input_event());
    testbed.push_text("oo");

    assert_eq!(testbed.context_menu().unwrap().editbox().contents(), "oo".to_string());

    assert!(testbed.has_items(["menu1", "option1", "option2"].into_iter()));
    assert!(testbed.has_none_of_items(["submenu", "child1", "child2"].into_iter()));

    testbed.push_input(Keycode::Backspace.to_key().to_input_event());
    testbed.push_input(Keycode::Backspace.to_key().to_input_event());

    assert_eq!(testbed.context_menu().unwrap().editbox().contents(), "".to_string());

    testbed.push_text("cd");

    assert_eq!(testbed.context_menu().unwrap().editbox().contents(), "cd".to_string());

    assert!(testbed.has_items(["menu1", "submenu", "child1", "child2"].into_iter()));
    assert!(testbed.has_none_of_items(["option1", "option2"].into_iter()));
    testbed.screenshot();
}
