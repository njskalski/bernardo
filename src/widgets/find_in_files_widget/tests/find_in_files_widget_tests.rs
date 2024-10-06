use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::io::keys::Keycode;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::spath;
use crate::widgets::find_in_files_widget::tests::find_in_files_widget_testbed::{
    AdditionalData, FindInFilesWidgetTestbed, FindInFilesWidgetTestbedBuilder, Msg,
};

pub fn get_setup_1() -> FindInFilesWidgetTestbed {
    // logger_setup(true, None, None);

    let mut setup = FindInFilesWidgetTestbedBuilder::new(AdditionalData {
        root: spath!(MockFS::new("./test_envs/find_in_files_test_1").to_fsf()).unwrap(),
    })
    .build();

    setup.next_frame();
    setup
}

#[test]
fn find_in_files_escape_cancels() {
    // this test is temporary, just used to see how the widget looks like
    let mut f = get_setup_1();

    debug_assert!(f.wait_for(|f| f.interpreter().is_some()));
    debug_assert!(f.interpreter().unwrap().search_button().is_focused());

    f.send_input(Keycode::Esc.to_key().to_input_event());

    debug_assert!(f.wait_for(|f| f.last_msg_as() == Some(&Msg::Cancel)));
}

#[test]
fn start_typing_jumps_to_query() {
    // this test is temporary, just used to see how the widget looks like
    let mut f = get_setup_1();

    debug_assert!(f.wait_for(|f| f.interpreter().is_some()));
    debug_assert!(f.interpreter().unwrap().search_button().is_focused());
    debug_assert!(f.interpreter().unwrap().query_box().is_focused() == false);
    debug_assert!(f.interpreter().unwrap().query_box().contents().is_empty());

    f.type_in("somequery");

    debug_assert!(f.wait_for(|f| f.interpreter().unwrap().query_box().contents() == "somequery"));
    debug_assert!(f.interpreter().unwrap().query_box().is_focused() == true);
}

// TODO test that find in files searches only in subtree

#[test]
fn find_in_files_dev_1() {
    // this test is temporary, just used to see how the widget looks like
    // let mut testbed = get_setup_1();
    //
    // testbed.next_frame();
    // testbed.interpreter();
    //
    // testbed.screenshot();
}
