use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::primitives::common_edit_msgs::CommonEditMsg::*;
use crate::text::buffer_state_fuzz::fuzz_call;

#[test]
fn crash_1() {
    let text = "\t".to_string();
    let msgs: Vec<CommonEditMsg> = vec![
        PageDown { selecting: true },
        ShiftTab,
        PageDown { selecting: true },
        WordBegin { selecting: true },
        Tab,
        Tab,
        Tab,
        Tab,
        Char('碌'),
    ];

    fuzz_call(text, msgs);
}
