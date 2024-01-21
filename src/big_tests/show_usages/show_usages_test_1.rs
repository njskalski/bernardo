use std::thread::sleep;
use std::time::Duration;

use crate::io::keys::Keycode;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::mock_navcomp_provider::MockSymbolMatcher;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::spath;
use crate::w7e::navcomp_provider::{NavCompSymbol, SymbolType, SymbolUsage};

fn get_full_setup() -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/show_usages_test_1")
        .with_files(["src/main.rs"])
        // .with_frame_based_wait()
        .build();

    {
        let mut symbols = full_setup.navcomp_pilot().unwrap().symbols().unwrap();

        let first_occ = (StupidCursor::new(4, 7), StupidCursor::new(17, 7));
        let second_occ = (StupidCursor::new(4, 11), StupidCursor::new(17, 11));

        let mockfs = full_setup.fsf();

        symbols.push(MockSymbolMatcher {
            path: spath!(mockfs, "src", "main.rs"),
            symbol: NavCompSymbol {
                symbol_type: SymbolType::Function,
                stupid_range: first_occ,
            },
            usages: Some(vec![
                SymbolUsage {
                    path: "src/main.rs".to_string(),
                    stupid_range: first_occ,
                },
                SymbolUsage {
                    path: "src/main.rs".to_string(),
                    stupid_range: second_occ,
                },
            ]),
        });
        symbols.push(MockSymbolMatcher {
            path: spath!(mockfs, "src", "main.rs"),
            symbol: NavCompSymbol {
                symbol_type: SymbolType::Function,
                stupid_range: second_occ,
            },
            usages: Some(vec![
                SymbolUsage {
                    path: "src/main.rs".to_string(),
                    stupid_range: first_occ,
                },
                SymbolUsage {
                    path: "src/main.rs".to_string(),
                    stupid_range: second_occ,
                },
            ]),
        });
    }

    full_setup
}

#[test]
fn show_usages_integ_test_1() {
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

    for _ in 0..7 {
        assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    }

    assert!(full_setup.wait_for(|f| f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .find(|line| line.visible_idx == 8)
        .is_some()));

    assert!(full_setup.send_key(Keycode::ArrowRight.to_key().with_ctrl()));

    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_lines()
            .find(|line| line.contents.text.trim() == "some_function(\"a\");⏎")
            .is_some()
    }));

    // TODO(#24)
    sleep(Duration::from_millis(300));

    full_setup.send_key(full_setup.config().keyboard_config.global.everything_bar);

    assert!(full_setup.wait_for(|f| { f.get_first_editor().unwrap().context_bar_op().is_some() }));

    assert!(full_setup.send_key(Keycode::ArrowDown.to_key()));
    assert!(full_setup.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .context_bar_op()
            .map(|c| c.selected_option().map(|c| c.trim().starts_with("show usages")).unwrap_or(false))
            .unwrap_or(false)
    }));

    assert!(full_setup.send_key(Keycode::Enter.to_key()));

    assert!(full_setup.wait_for(|full_setup| { full_setup.get_code_results_view().is_some() }));

    full_setup.wait_frame();
    // full_setup.screenshot();
}
