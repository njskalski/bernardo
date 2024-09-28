use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::spath;
use crate::widgets::find_in_files_widget::tests::find_in_files_widget_testbed::{AdditionalData, FindInFilesWidgetTestbed, FindInFilesWidgetTestbedBuilder};

mod find_in_files_widget_testbed;
pub fn get_setup_1() -> FindInFilesWidgetTestbed {
    FindInFilesWidgetTestbedBuilder::new(AdditionalData {
        root: spath!(MockFS::new("./test_envs/find_in_files_test_1").to_fsf()).unwrap(),
    }).build()
}

// #[test]
// fn context_menu_1_enter_expands() {
//     let mut testbed = get_setup_1();
//
//     testbed.next_frame();
//
//     assert_eq!(testbed.context_menu().unwrap().tree_view().items().len(), 1);
//     assert!(testbed.has_items(["menu1"].into_iter()));
//
//     testbed.push_input(Keycode::Enter.to_key().to_input_event());
//
//     assert!(testbed.wait_for(|testbed| { testbed.has_items(["submenu"].into_iter()) }));
// }
