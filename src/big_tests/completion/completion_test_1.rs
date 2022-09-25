use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::mocks::full_setup::{FullSetup, FullSetupBuilder};
use crate::spath;

#[test]
fn completion_test_1() {
    let mut full_setup: FullSetup = FullSetupBuilder::new("./test_envs/completion_test_1")
        .with_files(["src/main.rs"])
        .build();

    let file = spath!(full_setup.fsf(), "src", "main.rs").unwrap();

    assert!(full_setup.wait_frame());
    assert!(full_setup.is_editor_opened());
    assert!(full_setup.navcomp_pilot().wait_for_load(&file).is_some());

    assert_eq!(full_setup.focused_cursors().map(|x| x.0.y).collect::<Vec<u16>>(), vec![0]);

    for _ in 0..4 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| f.focused_cursor_lines().next().unwrap().1.contains("5")));

    assert!(full_setup.type_in("path."));
    assert!(full_setup.wait_for(|f| f.focused_cursor_lines().next().unwrap().1.contains("path.")));
    assert!(full_setup.send_key(Keycode::Space.to_key().with_ctrl()));
    assert!(full_setup.wait_frame());

    let end = full_setup.finish();
    assert!(end.screenshot());
}