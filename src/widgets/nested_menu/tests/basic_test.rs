use crate::widgets::nested_menu::tests::nested_menu_testbed::NestedMenuTestbed;

pub fn get_setup() -> NestedMenuTestbed {
    let editor_view_testbed = NestedMenuTestbed::new();

    editor_view_testbed
}

#[test]
fn nested_menu_1() {
    let mut testbed = NestedMenuTestbed::new();
}