#![no_main]

use bernardo::primitives::common_edit_msgs::CommonEditMsg;
use bernardo::text::buffer_state::BufferState;
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
            buffer_state.initialize_for_widget(1, None);

            for cem in cems.iter() {
                buffer_state.apply_common_edit_message(cem.clone(), 1, 10, None, false);
            }
        },
    }
});
