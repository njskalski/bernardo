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

#[test]
fn crash_2() {
    let text = "}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{11}\u{18}\u{18}\u{18}\u{18}\u{18}\rzzw".to_string();
    let msgs: Vec<CommonEditMsg> = vec![
        Block("\u{11}\u{11}\u{11}\u{11}\u{11}\0\0\u{1b}`\0\u{8}".to_string()),
        Char('\u{114c2}'),
        Block("\u{11}\u{11}\u{11}\u{18}\u{18}\u{18}\u{18}\u{18}".to_string()),
        WordEnd { selecting: true },
        SubstituteBlock {
            char_range: 0..46,
            with_what: "".to_string(),
        },
    ];

    fuzz_call(text, msgs);
}
