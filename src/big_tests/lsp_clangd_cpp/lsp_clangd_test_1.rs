use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;

fn get_full_setup(file: &str) -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/lsp_clangd_cpp_1")
        .with_files([file])
        .with_mock_navcomp(false)
        // .with_frame_based_wait()
        .build();

    full_setup
}

#[test]
fn completions_clangd_cpp_completion() {
    let mut full_setup = get_full_setup("src/main.cpp");
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
        .any(|line| line.visible_idx == 10)));

    assert!(full_setup.send_key(Keycode::Tab.to_key()));

    assert!(full_setup.type_in("some."));

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines()
            .any(|line| line.contents.text.trim() == "some.⏎")
    }));

    assert!(full_setup.send_key(Keycode::Space.to_key().with_ctrl()));

    assert!(full_setup.wait_for(|f| { f.get_first_editor().unwrap().completions().map(|comp| comp.is_loading()) == Some(false) }));

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
            .any(|item| item.text.contains(infix)));
    }

    // full_setup.wait_frame();
    // full_setup.screenshot();
}

#[test]
fn highlighting_clangd_cpp_header() {
    let mut full_setup = get_full_setup("src/hello.hpp");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // moving cursor to empty line so it does not interfere with highlighting
    for _ in 0..1 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .any(|line| line.visible_idx == 2)));

    let vec: Vec<_> = full_setup.get_first_editor().unwrap().consistent_items_iter().collect();

    // let x =

    assert_eq!(vec[0].text.as_str(), "#include");
    assert_eq!(
        vec[0].text_style.foreground,
        full_setup.get_theme().name_to_color("keyword.import").unwrap()
    );

    assert_eq!(vec[1].text.as_str(), " <vector>");
    assert_eq!(
        vec[1].text_style.foreground,
        full_setup.get_theme().name_to_color("string").unwrap()
    );

    // TODO this test is incomplete, it's here because it tests "something", which is better than nothing
}

#[test]
fn highlighting_clangd_cpp_file() {
    let mut full_setup = get_full_setup("src/main.cpp");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));

    // moving cursor to empty line so it does not interfere with highlighting
    for _ in 0..2 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .any(|line| line.visible_idx == 2)));

    let vec: Vec<_> = full_setup.get_first_editor().unwrap().consistent_items_iter().collect();

    assert_eq!(
        vec.iter()
            .find(|item| item.text.contains("#include"))
            .expect("no includes found")
            .text_style
            .foreground,
        full_setup.get_theme().name_to_color("keyword.import").unwrap()
    );

    assert_eq!(
        vec.iter()
            .find(|item| item.text.contains("<cstdio>"))
            .expect("no <cstdio> found")
            .text_style
            .foreground,
        full_setup.get_theme().name_to_color("string").unwrap()
    );

    assert_eq!(
        vec.iter().find(|item| item.text.contains("return")).unwrap().text_style.foreground,
        full_setup.get_theme().name_to_color("keyword.return").unwrap()
    );

    // TODO this test is incomplete, it's here because it tests "something", which is better than nothing
}
