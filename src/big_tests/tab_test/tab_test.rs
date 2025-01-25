use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/tab_test_1")
        .with_files(["file_with_tabs.txt"].iter())
        .build();

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup
}

#[test]
fn tab_test_1() {
    let mut f = common_start();

    let lines: Vec<_> = f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines_with_coded_cursors()
        .collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].contents.text.as_str(), "#\ttab⏎")
}

#[test]
fn tab_test_2() {
    let mut f = common_start();

    let lines: Vec<_> = f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines_with_coded_cursors()
        .collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].contents.text.as_str(), "#\ttab⏎");

    assert!(f.send_key(Keycode::End.to_key().with_shift()));

    assert!(f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines_with_coded_cursors()
            .map(|i| i.contents.text)
            .find(|i| {
                println!("{}", i);
                i == "(\ttab]⏎"
            })
            .is_some()
    }));
}
