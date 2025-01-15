use std::thread::{self, sleep};
use std::time::Duration;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::mocks::editor_interpreter::LineIdxTuple;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;

fn common_start() -> FullSetup {
    let mut f: FullSetup = FullSetup::new("./test_envs/scroll_code_res_hit_test")
        // .with_frame_based_wait()
        .build();

    assert!(f.wait_for(|f| f.is_no_editor_opened()));

    f
}

#[test]
fn find_in_files_opens() {
    let mut f = common_start();

    assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
    assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
    assert!(f.get_find_in_files().unwrap().is_focused());

    f.type_in("min");

    assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().query_box().contents().contains("min") }));

    f.send_input(Keycode::Enter.to_key().to_input_event());

    assert!(f.wait_for(|f| { f.get_code_results_view().is_some() }));

    // TODO remove tick
    sleep(Duration::from_secs(1));
    f.send_input(InputEvent::Tick);

    // displays

    f.send_input(Keycode::Enter.to_key().to_input_event());

    assert!(f.wait_for(|f| f.get_code_results_view().is_none()));
    assert!(f.wait_for(|f| { f.get_first_editor().is_some() }));

    let lines: Vec<LineIdxTuple> = f.get_first_editor().unwrap().get_all_visible_lines().collect();

    assert!(lines[0]
        .contents
        .text
        .starts_with("\"At vero eos et accusamus et iusto odio dignissimos"));
    assert!(lines[4]
        .contents
        .text
        .starts_with("placeat facere possimus, omnis voluptas assumenda"));

    f.screenshot();

    // Now perform scroll test
    assert!(f.wait_for(|f| f.is_editor_opened()));

    assert_eq!(
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .map(|c| c.visible_idx)
            .collect::<Vec<usize>>(),
        vec![4]
    );

    for _ in 0..50 {
        assert!(f.send_key(Keycode::ArrowDown.to_key()));
    }

    f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_line_indices()
            .next()
            .map(|cursor| cursor.visible_idx == 51)
            .unwrap_or(false)
    });

    assert!(f.send_key(Keycode::ArrowRight.to_key().with_ctrl()));

    f.wait_for(|f| {
        f.get_first_editor()
            .unwrap()
            .get_visible_cursor_cells()
            .next()
            .map(|item| item.1.grapheme() == Some("p"))
            .unwrap_or(false)
    });
    f.screenshot();

    f.send_key(f.config().keyboard_config.global.everything_bar);

    f.wait_for(|f| f.get_first_context_menu().is_some());

    let rect = f.get_first_context_menu().unwrap().meta().rect;
    assert!(Rect::from_zero(f.get_frame().unwrap().buffer.size()).contains_rect(rect));

    let cursor_pos = f
        .get_first_editor()
        .unwrap()
        .get_visible_cursor_lines()
        .next()
        .unwrap()
        .contents
        .absolute_pos;
    assert!(!rect.contains(cursor_pos));

    // f.screenshot();
}

// //Using test env from rust integration test
// fn get_f() -> FullSetup {
//     let f: FullSetup = FullSetup::new("./test_envs/scroll_code_res_hit_test")
//         .with_files(["src/main.rs"])
//         .with_mock_navcomp(false)
//         .with_timeout(Duration::from_secs(20))
//         .build();

//     f
// }

// // assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
// //     assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
// //     assert!(f.get_find_in_files().unwrap().is_focused());

// //     f.type_in("min");

// //     assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().query_box().contents().contains("min") }));

// //     f.send_input(Keycode::Enter.to_key().to_input_event());

// //     assert!(f.wait_for(|f| { f.get_code_results_view().is_some() }));

// //     // TODO remove tick
// //     sleep(Duration::from_secs(1));
// //     f.send_input(InputEvent::Tick);

// //     // displays
// //     assert!(f.wait_for(|f| { f.get_code_results_view().unwrap().editors().len() == 3 }));

// //     f.screenshot();

// //     assert!(f.wait_for(|f| {
// //         f.get_code_results_view()
// //             .unwrap()
// //             .editors()
// //             .first()
// //             .map(|editor| {
// //                 editor
// //                     .get_visible_cursor_lines()
// //                     .find(|line| line.contents.text.starts_with("distinctio. Nam libero tempore"))
// //                     .is_some()
// //             })
// //             .unwrap_or(false)
// //     }));

// #[test]
// fn go_to_definition_and_scroll_test() {
//     if std::env::var("CI").is_ok() {
//         return;
//     }

//     let mut f = get_f();
//     assert!(f.wait_for(|f| f.is_editor_opened()));

//     assert!(f.send_key(f.config().keyboard_config.global.find_in_files));
//     assert!(f.wait_for(|f| f.get_find_in_files().is_some()));
//     assert!(f.get_find_in_files().unwrap().is_focused());

//     f.type_in("some_function");

//     assert!(f.wait_for(|f| { f.get_find_in_files().unwrap().query_box().contents().contains("some_function") }));
//     f.screenshot();

//     f.send_input(Keycode::Enter.to_key().to_input_event());

//     assert!(f.wait_for(|f| { f.get_code_results_view().is_some() }));
//     f.screenshot();
//     // return;
//     // TODO remove tick
//     sleep(Duration::from_secs(1));
//     f.send_input(InputEvent::Tick);

//     // displays
//     assert!(f.wait_for(|f| { f.get_code_results_view().is_some() }));

//     f.screenshot();
//     return;

//     // assert!(f.wait_for(|f| {
//     //     f.get_code_results_view()
//     //         .unwrap()
//     //         .editors()
//     //         .first()
//     //         .map(|editor| {
//     //             editor
//     //                 .get_visible_cursor_lines()
//     //                 .find(|line| line.contents.text.starts_with("distinctio. Nam libero tempore"))
//     //                 .is_some()
//     //         })
//     //         .unwrap_or(false)
//     // }));

//     assert_eq!(
//       f
//           .get_first_editor()
//           .unwrap()
//           .get_visible_cursor_line_indices()
//           .map(|c| c.visible_idx)
//           .collect::<Vec<usize>>(),
//       vec![1]
//   );

//   for _ in 0..50 {
//       assert!(f.send_key(Keycode::ArrowDown.to_key()));
//   }

//   f.wait_for(|f| {
//       f
//           .get_first_editor()
//           .unwrap()
//           .get_visible_cursor_line_indices()
//           .next()
//           .map(|cursor| cursor.visible_idx == 51)
//           .unwrap_or(false)
//   });

//   assert!(f.send_key(Keycode::ArrowRight.to_key().with_ctrl()));

//   f.wait_for(|f| {
//       f
//           .get_first_editor()
//           .unwrap()
//           .get_visible_cursor_cells()
//           .next()
//           .map(|item| item.1.grapheme() == Some("p"))
//           .unwrap_or(false)
//   });

//   f.send_key(f.config().keyboard_config.global.everything_bar);

//   f.wait_for(|f| f.get_first_context_menu().is_some());

//   let rect = f.get_first_context_menu().unwrap().meta().rect;
//   assert!(Rect::from_zero(f.get_frame().unwrap().buffer.size()).contains_rect(rect));

//   let cursor_pos = f
//       .get_code_results_view()
//       .unwrap()
//       .editors()
//       .first()
//       .unwrap()
//       .get_visible_cursor_cells()
//       .next()
//       .unwrap()
//       .0;
//   assert!(!rect.contains(cursor_pos));

// }
