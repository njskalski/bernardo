#![no_main]

use bernardo::primitives::common_edit_msgs::CommonEditMsg;
use bernardo::primitives::cursor_set::CursorSet;
use bernardo::primitives::cursor_set_fuzz;
use bernardo::primitives::cursor_set_fuzz::*;
use bernardo::text::buffer_state::BufferState;
use bernardo::text::buffer_state_fuzz::*;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(arbitrary::Arbitrary, Debug)]
struct Items {
    buffer_state: BufferState,
    cems: Vec<CommonEditMsg>,
}

fuzz_target!(|items : Items| {
    match items {
        Items { mut buffer_state, cems } => {
            for cem in cems.iter() {
                buffer_state.apply_cem(cem.clone(), 10, None);
            }
        },
    }
});
