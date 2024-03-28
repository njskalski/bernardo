use crate::experiments::screen_shot::screenshot;
use crate::io::keys::Keycode;
use crate::mocks::mock_tree_item::get_mock_data_set_1;
use crate::widgets::nested_menu::tests::nested_menu_testbed::{NestedMenuTestMsg, NestedMenuTestbed};

pub fn get_setup_1() -> NestedMenuTestbed {
    let nested_menu_testbed = NestedMenuTestbed::new(get_mock_data_set_1());

    nested_menu_testbed
}

#[test]
fn nested_menu_1() {
    let mut testbed = get_setup_1();

    testbed.next_frame();
    assert_eq!(testbed.frame_op().unwrap().get_nested_menus().count(), 1);

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    let items = testbed.nested_menu().unwrap().get_items().collect::<Vec<_>>();

    assert_eq!(
        items.iter().map(|item| item.label.clone()).collect::<Vec<_>>(),
        vec!["option1".to_string(), "option2".to_string(), "submenu".to_string()]
    );

    assert_eq!(items.iter().map(|item| item.leaf).collect::<Vec<_>>(), vec![true, true, false]);
}

#[test]
fn nested_menu_2_arrows_up_noop() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    testbed.push_input(Keycode::ArrowUp.to_key().to_input_event());

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );
}

#[test]
fn nested_menu_3_arrow_down() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option2".to_string()
    );
}

#[test]
fn nested_menu_4_arrow_down_noop() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    for _ in 0..2 {
        testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());
    }

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "submenu".to_string()
    );

    testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "submenu".to_string()
    );
}

#[test]
fn nested_menu_5_enter_expands() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    let x = testbed.nested_menu().unwrap().get_selected_item().unwrap().label;

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    for _ in 0..2 {
        testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());
    }

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "submenu".to_string()
    );

    // screenshot(&testbed.last_frame.clone().unwrap().buffer);

    testbed.push_input(Keycode::Enter.to_key().to_input_event());

    let items = testbed.nested_menu().unwrap().get_items().collect::<Vec<_>>();

    assert_eq!(items[0].label, "submenu".to_string());
    assert_eq!(items[1].label, "child1".to_string());
    assert_eq!(items[2].label, "child2".to_string());

    assert_eq!(items[0].leaf, false);
    assert_eq!(items[1].leaf, true);
    assert_eq!(items[2].leaf, true);

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "child1".to_string()
    );

    screenshot(&testbed.last_frame.unwrap().buffer);
}

#[test]
fn nested_menu_6_arrow_right_expands() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    for _ in 0..2 {
        testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());
    }

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "submenu".to_string()
    );

    testbed.push_input(Keycode::ArrowRight.to_key().to_input_event());

    let items = testbed.nested_menu().unwrap().get_items().collect::<Vec<_>>();

    assert_eq!(items[0].label, "submenu".to_string());
    assert_eq!(items[1].label, "child1".to_string());
    assert_eq!(items[2].label, "child2".to_string());

    assert_eq!(items[0].leaf, false);
    assert_eq!(items[1].leaf, true);
    assert_eq!(items[2].leaf, true);

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "child1".to_string()
    );
}

#[test]
fn nested_menu_7_arrow_left_collapses() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    for _ in 0..2 {
        testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());
    }

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "submenu".to_string()
    );

    testbed.push_input(Keycode::ArrowRight.to_key().to_input_event());

    {
        let items = testbed.nested_menu().unwrap().get_items().collect::<Vec<_>>();

        assert_eq!(items[0].label, "submenu".to_string());
        assert_eq!(items[1].label, "child1".to_string());
        assert_eq!(items[2].label, "child2".to_string());

        assert_eq!(items[0].leaf, false);
        assert_eq!(items[1].leaf, true);
        assert_eq!(items[2].leaf, true);
    }

    testbed.push_input(Keycode::ArrowLeft.to_key().to_input_event());

    testbed.next_frame();

    {
        let items = testbed.nested_menu().unwrap().get_items().collect::<Vec<_>>();

        assert_eq!(items[0].label, "option1".to_string());
        assert_eq!(items[1].label, "option2".to_string());
        assert_eq!(items[2].label, "submenu".to_string());

        assert_eq!(items[0].leaf, true);
        assert_eq!(items[1].leaf, true);
        assert_eq!(items[2].leaf, false);
    }
}

#[test]
fn nested_menu_8_msgs() {
    let mut testbed = get_setup_1();

    testbed.next_frame();

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "option1".to_string()
    );

    testbed.push_input(Keycode::Enter.to_key().to_input_event());

    assert_eq!(
        testbed.last_msg.take().unwrap().as_msg::<NestedMenuTestMsg>(),
        Some(NestedMenuTestMsg::Text("option1".to_string())).as_ref()
    );

    for _ in 0..2 {
        testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());
    }

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "submenu".to_string()
    );

    testbed.push_input(Keycode::Enter.to_key().to_input_event());
    assert!(testbed.last_msg.is_none());

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "child1".to_string()
    );

    testbed.push_input(Keycode::Enter.to_key().to_input_event());

    assert_eq!(
        testbed.last_msg.take().unwrap().as_msg::<NestedMenuTestMsg>(),
        Some(NestedMenuTestMsg::Text("child1".to_string())).as_ref()
    );

    testbed.push_input(Keycode::ArrowDown.to_key().to_input_event());

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "child2".to_string()
    );

    testbed.push_input(Keycode::Enter.to_key().to_input_event());

    assert_eq!(
        testbed.last_msg.take().unwrap().as_msg::<NestedMenuTestMsg>(),
        Some(NestedMenuTestMsg::Text("child2".to_string())).as_ref()
    );

    testbed.push_input(Keycode::ArrowLeft.to_key().to_input_event());

    assert_eq!(
        testbed.nested_menu().unwrap().get_selected_item().unwrap().label,
        "submenu".to_string()
    );
}
