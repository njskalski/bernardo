use std::path::PathBuf;

use crate::mocks::full_setup::FullSetup;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/save_test_1")
        .with_files(["src/main.rs"])
        .build();

    assert!(full_setup.wait_frame());

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    full_setup
}


#[test]
fn save_saves() {
    let mut full_setup = common_start();
    let test_string = "this_is_a_test_string";

    // no string on screen
    assert!(!full_setup.get_first_editor().unwrap().get_all_lines().fold(false, |acc, line| {
        acc | line.contents.text.contains(test_string)
    }));

    let file = full_setup.fsf().root().descendant_checked("src/main.rs");
    assert!(file.is_some());
    let file = file.unwrap();

    // no string present in file
    assert!(!file.read_entire_file_to_string().unwrap().contains(test_string));

    // we type it in
    full_setup.type_in(test_string);

    // it appeared on screen
    assert!(
        full_setup.wait_for(
            |full_setup| full_setup.get_first_editor().unwrap().get_all_lines().fold(false, |acc, line| {
                acc | line.contents.text.contains(test_string)
            })));

    // we hit ctrl-s
    assert!(full_setup.send_key(full_setup.config().keyboard_config.editor.save));

    full_setup.screenshot();

    // and now the filesystem DOES contain the string in question
    let read_back = file.read_entire_file_to_string().unwrap();
    assert!(read_back.contains(test_string));

    full_setup.finish();
}
