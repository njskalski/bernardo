use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::spath;

fn common_start() -> FullSetup {
    let mut full_setup: FullSetup = FullSetup::new("./test_envs/buffer_list_test_1")
        // .with_frame_based_wait()
        .build();

    assert!(full_setup.wait_for(|f| f.is_no_editor_opened()));

    let files_to_open: Vec<_> = vec!["data11", "data22", "data33"];

    for file in files_to_open {
        full_setup.send_key(full_setup.config().keyboard_config.global.fuzzy_file);
        assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));
        full_setup.type_in(file);
        full_setup.type_in(".txt");

        assert!(full_setup.wait_for(|full_setup| {
            let fs = full_setup.get_fuzzy_search().unwrap();
            let sel = fs.selected_option();
            sel.map(|o| o.contains(&(file.to_string() + ".txt"))).unwrap_or(false)
        }));

        full_setup.send_key(Keycode::Enter.to_key());

        assert!(full_setup.wait_for(|full_setup| { full_setup.get_first_editor().is_some() }));
        assert!(full_setup.wait_for(|full_setup| {
            full_setup
                .get_first_editor()
                .unwrap()
                .get_visible_cursor_lines()
                .next()
                .map(|item| item.contents.text.contains(file))
                .unwrap_or(false)
        }))
    }

    full_setup
}

#[test]
fn buffer_list_hit_opens_file_and_closes_the_list() {
    let mut full_setup = common_start();

    full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
    assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

    full_setup.type_in("data11.txt");

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().unwrap().editbox().contents().contains("data11.txt") }));

    full_setup.send_input(Keycode::Enter.to_key().to_input_event());

    assert!(full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_all_visible_lines()
            .find(|line| line.contents.text.contains("data11"))
            .is_some()
    }));
    assert!(full_setup.get_fuzzy_search().is_none());
}

#[test]
fn non_edited_files_have_no_markers_1() {
    let mut full_setup = common_start();

    full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
    assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

    let all: Vec<String> = full_setup
        .get_fuzzy_search()
        .unwrap()
        .visible_items()
        .iter()
        .filter(|item| item.leaf)
        .map(|item| item.label.clone())
        .collect();

    assert_eq!(all.len(), 3);

    for name in all.iter() {
        assert_eq!(name.contains("[*]"), false);
    }
}

#[test]
fn edited_files_have_markers() {
    let mut full_setup = common_start();

    full_setup.type_in("sometext");
    full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_all_visible_lines()
            .find(|item| item.contents.text.contains("sometext"))
            .is_some()
    });

    full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
    assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

    let all: Vec<String> = full_setup
        .get_fuzzy_search()
        .unwrap()
        .visible_items()
        .iter()
        .filter(|item| item.leaf)
        .map(|item| item.label.clone())
        .collect();

    assert_eq!(all.len(), 3);

    for name in all.iter() {
        assert_eq!(name.contains("[*]"), name.contains("data33"));
    }
}

#[test]
fn saving_removes_markers() {
    let mut full_setup = common_start();

    full_setup.type_in("sometext");
    full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_all_visible_lines()
            .find(|item| item.contents.text.contains("sometext"))
            .is_some()
    });

    // edited - mark on data33, no marks on others
    {
        full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
        assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

        let all: Vec<String> = full_setup
            .get_fuzzy_search()
            .unwrap()
            .visible_items()
            .iter()
            .filter(|item| item.leaf)
            .map(|item| item.label.clone())
            .collect();

        assert_eq!(all.len(), 3);

        for name in all.iter() {
            assert_eq!(name.contains("[*]"), name.contains("data33"));
        }
    }
    full_setup.send_key(Keycode::Esc.to_key());
    assert!(full_setup.wait_for(|full_setup| full_setup.get_fuzzy_search().is_none()));

    full_setup.send_key(full_setup.config().keyboard_config.editor.save);

    assert!(full_setup.wait_for(|full_setup| -> bool {
        let file = match spath!(full_setup.fsf(), "data33.txt") {
            None => {
                return false;
            }
            Some(f) => f,
        };

        if let Some(content_string) = file.read_entire_file_to_string().ok() {
            content_string.contains("sometext")
        } else {
            false
        }
    }));

    // saved - mark removed
    {
        full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
        assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

        let all: Vec<String> = full_setup
            .get_fuzzy_search()
            .unwrap()
            .visible_items()
            .iter()
            .filter(|item| item.leaf)
            .map(|item| item.label.clone())
            .collect();

        assert_eq!(all.len(), 3);

        for name in all.iter() {
            assert_eq!(name.contains("[*]"), false);
        }
    }
}

#[test]
fn purge_buffers_works() {
    let mut full_setup = common_start();

    full_setup.type_in("sometext");
    full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_all_visible_lines()
            .find(|item| item.contents.text.contains("sometext"))
            .is_some()
    });

    // edited - mark on data33, no marks on others
    {
        full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
        assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

        let all: Vec<String> = full_setup
            .get_fuzzy_search()
            .unwrap()
            .visible_items()
            .iter()
            .filter(|item| item.leaf)
            .map(|item| item.label.clone())
            .collect();

        assert_eq!(all.len(), 3);

        for name in all.iter() {
            assert_eq!(name.contains("[*]"), name.contains("data33"));
        }
    }
    full_setup.send_key(Keycode::Esc.to_key());
    assert!(full_setup.wait_for(|full_setup| full_setup.get_fuzzy_search().is_none()));

    full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar);

    assert!(full_setup.wait_for(|full_setup| full_setup.get_first_context_menu().is_some()));

    full_setup.type_in("prune");
    for _ in 0..2 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
    }

    assert!(full_setup.wait_for(|full_setup| full_setup
        .get_first_context_menu()
        .unwrap()
        .selected_option()
        .unwrap()
        .contains("prune")));
    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| full_setup.get_first_context_menu().is_none()));

    // now only data33.txt should remain open, as all others were pruned
    {
        full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
        assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

        let all: Vec<String> = full_setup
            .get_fuzzy_search()
            .unwrap()
            .visible_items()
            .iter()
            .filter(|item| item.leaf)
            .map(|item| item.label.clone())
            .collect();

        assert_eq!(all, vec!["data33.txt [*]"]);
    }
    full_setup.screenshot();
}

#[test]
fn close_buffer_test() {
    let mut full_setup = common_start();

    full_setup.type_in("sometext");
    full_setup.wait_for(|full_setup| {
        full_setup
            .get_first_editor()
            .unwrap()
            .get_all_visible_lines()
            .find(|item| item.contents.text.contains("sometext"))
            .is_some()
    });

    full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar);

    assert!(full_setup.wait_for(|full_setup| full_setup.get_first_context_menu().is_some()));

    full_setup.type_in("close");
    for _ in 0..2 {
        full_setup.send_key(Keycode::ArrowDown.to_key());
    }

    assert!(full_setup.wait_for(|full_setup| full_setup
        .get_first_context_menu()
        .unwrap()
        .selected_option()
        .unwrap()
        .contains("close")));
    full_setup.send_key(Keycode::Enter.to_key());

    assert!(full_setup.wait_for(|full_setup| full_setup.get_first_context_menu().is_none()));

    // now only data33.txt should remain open, as all others were pruned
    {
        full_setup.send_key(full_setup.config().keyboard_config.global.browse_buffers);
        assert!(full_setup.wait_for(|full_setup| { full_setup.get_fuzzy_search().is_some() }));

        let all: Vec<String> = full_setup
            .get_fuzzy_search()
            .unwrap()
            .visible_items()
            .iter()
            .filter(|item| item.leaf)
            .map(|item| item.label.clone())
            .collect();

        full_setup.screenshot();
        assert_eq!(all, vec!["data33.txt [*]"]);
    }
}
