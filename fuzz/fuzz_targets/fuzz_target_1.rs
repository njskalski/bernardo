#![no_main]
#[macro_use]
extern crate libfuzzer_sys;

use bernardo::config::config::KeyboardConfig;
use bernardo::io::keys::{Key, Keycode};
use bernardo::mocks::fuzz_call::fuzz_call;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    println!("fuzz1: len {}", data.len());
    let options = key_list();
    let oplen = options.len();

    let inputs : Vec<Key> = data.iter().map(move |byte| {
        let idx = (*byte as usize) % oplen;
        let key = &options[idx].clone();
        key.clone()
    }).collect();

    println!("will use input : {:?}", inputs);

    fuzz_call(inputs);
});



fn key_list() -> Vec<Key> {
    let mut result: Vec<Key> = vec![];

    for i in 0..26 {
        let char = ('a' as u8 + i as u8) as char;
        let k = Keycode::Char(char);
        result.push(k.to_key());
    }

    let config = KeyboardConfig::default();

    let mut x = vec![
        Keycode::ArrowUp.to_key(),
        Keycode::ArrowUp.to_key().with_shift(),
        Keycode::ArrowDown.to_key(),
        Keycode::ArrowDown.to_key().with_shift(),
        Keycode::ArrowLeft.to_key(),
        Keycode::ArrowLeft.to_key().with_shift(),
        Keycode::ArrowRight.to_key(),
        Keycode::ArrowRight.to_key().with_shift(),
        Keycode::Enter.to_key(),
        Keycode::Enter.to_key().with_shift(),
        Keycode::Space.to_key(),
        Keycode::Space.to_key().with_shift(),
        Keycode::Space.to_key().with_ctrl(),
        Keycode::LeftAlt.to_key(),
        Keycode::LeftAlt.to_key().with_shift(),
        Keycode::RightAlt.to_key(),
        Keycode::RightAlt.to_key().with_shift(),
        Keycode::LeftCtrl.to_key(),
        Keycode::LeftCtrl.to_key().with_shift(),
        Keycode::RightCtrl.to_key(),
        Keycode::RightCtrl.to_key().with_shift(),
        Keycode::Backspace.to_key(),
        Keycode::Backspace.to_key().with_shift(),
        Keycode::Home.to_key(),
        Keycode::Home.to_key().with_shift(),
        Keycode::End.to_key(),
        Keycode::End.to_key().with_shift(),
        Keycode::PageUp.to_key(),
        Keycode::PageUp.to_key().with_shift(),
        Keycode::PageDown.to_key(),
        Keycode::PageDown.to_key().with_shift(),
        Keycode::Tab.to_key(),
        Keycode::Tab.to_key().with_shift(),
        Keycode::Delete.to_key(),
        Keycode::Delete.to_key().with_shift(),
        Keycode::Insert.to_key(),
        Keycode::Insert.to_key().with_shift(),
        // Keycode::Null.to_key(),
        // Keycode::Null.to_key().with_shift(),
        Keycode::Esc.to_key(),
        Keycode::Esc.to_key().with_shift(),
        config.global.quit,
        config.global.fuzzy_file,
        config.global.new_buffer,
        config.global.close_buffer,
        config.global.browse_buffers,
        config.global.everything_bar,
        config.global.find_in_files,
        config.global.close_context_menu,
        config.global.next_display,
        config.global.prev_display,
        config.editor.save,
        config.editor.save_as,
        config.editor.enter_cursor_drop_mode,
        config.editor.find,
        config.editor.close_find_replace,
        config.editor.request_completions,
        config.editor.reformat,
        config.file_tree.toggle_hidden_files,
        config.edit_msgs.tab,
        config.edit_msgs.shift_tab,
        config.edit_msgs.backspace,
        config.edit_msgs.line_begin,
        config.edit_msgs.line_end,
        config.edit_msgs.word_begin,
        config.edit_msgs.word_end,
        config.edit_msgs.copy,
        config.edit_msgs.paste,
        config.edit_msgs.undo,
        config.edit_msgs.redo,
        config.edit_msgs.home,
    ];

    result.append(&mut x);

    debug_assert!(result.len() < 256);

    result
}
