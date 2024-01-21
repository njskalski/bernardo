use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;

fn get_full_setup() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/lsp_clangd_1")
        .with_files(["src/main.cpp"])
        .with_mock_navcomp(false)
        // .with_frame_based_wait()
        .build();

    full_setup
}

#[test]
fn completions_clangd_integ_test_1() {
    let mut full_setup = get_full_setup();
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    assert_eq!(
        full_setup
            .get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .map(|c| c.visible_idx)
            .collect::<Vec<usize>>(),
        vec![1]
    );

    for _ in 0..10 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .find(|line| line.visible_idx == 10)
        .is_some()));

    assert!(full_setup.send_key(Keycode::Tab.to_key()));

    assert!(full_setup.type_in("some."));

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines()
            .find(|line| line.contents.text.trim() == "some.⏎")
            .is_some()
    }));

    assert!(full_setup.send_key(Keycode::Space.to_key().with_ctrl()));

    assert!(full_setup.wait_for(|f| { f.get_first_editor().unwrap().completions().map(|comp| comp.is_loading()) == Some(false) }));

    let items: Vec<String> = full_setup
        .get_first_editor()
        .unwrap()
        .completions()
        .unwrap()
        .items()
        .map(|item| item.text)
        .collect();

    // each infix should appear at least once
    let expected_infixes: Vec<&'static str> = vec!["assign(", "at(size_type n)", "back()", "begin()"];

    let expected_highlight = "assign(";

    assert!(full_setup
        .get_first_editor()
        .unwrap()
        .completions()
        .unwrap()
        .highlighted(true)
        .unwrap()
        .1
        .contains(expected_highlight));

    for infix in expected_infixes.iter() {
        assert!(full_setup
            .get_first_editor()
            .unwrap()
            .completions()
            .unwrap()
            .items()
            .find(|item| item.text.contains(infix))
            .is_some());
    }

    // full_setup.wait_frame();
    // full_setup.screenshot();
}
