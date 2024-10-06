use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::io::keys::Keycode;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::spath;
use crate::widgets::find_in_files_widget::tests::find_in_files_widget_testbed::{AdditionalData, FindInFilesWidgetTestbed, FindInFilesWidgetTestbedBuilder, Msg};

pub fn get_setup_1() -> FindInFilesWidgetTestbed {
    // logger_setup(true, None, None);

    FindInFilesWidgetTestbedBuilder::new(AdditionalData {
        root: spath!(MockFS::new("./test_envs/find_in_files_test_1").to_fsf()).unwrap(),
    })
        .build()
}

#[test]
fn find_in_files_escape_cancels() {
    // this test is temporary, just used to see how the widget looks like
    let mut f = get_setup_1();

    f.wait_for(|f| {
        f.interpreter().is_some()
    });

    f.send_input(Keycode::Esc.to_key().to_input_event());

    f.wait_for(|f| {
        f.last_msg_as() == Some(&Msg::Cancel)
    });
}

#[test]
fn find_in_files_dev_1() {
    // this test is temporary, just used to see how the widget looks like
    let mut testbed = get_setup_1();

    testbed.next_frame();
    testbed.interpreter();

    testbed.screenshot();
}
