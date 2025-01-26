#![no_main]
#[macro_use]
extern crate libfuzzer_sys;

use bernardo::config::config::Config;
use bernardo::io::keys::{Key, Keycode};
use bernardo::mocks::full_setup::FullSetup;
use bernardo::mocks::with_wait_for::WithWaitFor;

fn common_start() -> bernardo::mocks::full_setup::FullSetup {
    let mut config = Config::default();
    config.global.tabs_to_spaces = None;

    let mut full_setup: FullSetup = FullSetup::new("./test_envs/tab_test_1")
        .with_config(config)
        .build();

    full_setup.send_key(Keycode::Char('n').to_key());

    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
    assert!(full_setup.wait_for(|f| f.get_first_editor().unwrap().is_editor_focused()));

    full_setup
}

fn key_list() -> Vec<Key> {
    let mut result: Vec<Key> = vec![];

    for i in 0..26 {
        let char = ('a' as u8 + i as u8) as char;
        let k = Keycode::Char(char);
        result.push(k.to_key());
    }

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
    ];

    result.append(&mut x);

    debug_assert!(result.len() < 256);

    result
}


fuzz_target!(|data: &[u8]| {
    let mut f = common_start();
    let options = key_list();
    let oplen = options.len();

    for char in data {
        let idx = (*char as usize) % oplen;
        let key = &options[idx].clone();

        f.send_key(*key);
        f.wait_frame();
    }
});