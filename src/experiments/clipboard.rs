use std::cell::{ RefCell};
use std::rc::Rc;
use log::error;

const EMPTY_STRING: String = String::new();

pub type ClipboardRef = Rc<Box<dyn Clipboard>>;

pub trait Clipboard {
    fn get(&self) -> String;
    fn set(&self, s: String) -> bool;
}

pub fn get_me_some_clipboard() -> ClipboardRef {
    match DefaultClipboard::new() {
        Some(dc) => Rc::new(Box::new(dc)),
        None => {
            error!("using failsafe fake clipboard");
            Rc::new(Box::new(FakeClipboard::default()))
        }
    }
}

pub fn get_me_fake_clipboard() -> ClipboardRef {
    Rc::new(Box::new(FakeClipboard::default()))
}

struct DefaultClipboard {
    clipboard: RefCell<arboard::Clipboard>,
}

impl DefaultClipboard {
    pub fn new() -> Option<Self> {
        match arboard::Clipboard::new() {
            Ok(c) => Some(DefaultClipboard { clipboard: RefCell::new(c) }),
            Err(e) => {
                error!("failed acquiring clipboard: {:?}", e);
                None
            }
        }
    }
}

impl Clipboard for DefaultClipboard {
    fn get(&self) -> String {
        match self.clipboard.try_borrow_mut() {
            Ok(mut clipboard) => {
                match clipboard.get_text() {
                    Ok(text) => text,
                    Err(e) => {
                        error!("error getting text from clipboard: {:?}", e);
                        EMPTY_STRING
                    }
                }
            }
            Err(e) => {
                error!("failed acquiring clipboard lock: {:?}", e);
                EMPTY_STRING
            }
        }
    }

    fn set(&self, contents: String) -> bool {
        match self.clipboard.try_borrow_mut() {
            Ok(mut clipboard) => {
                clipboard.set_text(contents).map_err(|e| {
                    error!("error setting clipboard contents: {:?}", e)
                }).is_ok()
            }
            Err(e) => {
                error!("failed acquiring clipboard lock: {:?}", e);
                false
            }
        }
    }
}

struct FakeClipboard {
    contents: RefCell<String>,
}

impl Default for FakeClipboard {
    fn default() -> Self {
        FakeClipboard {
            contents: RefCell::new(EMPTY_STRING),
        }
    }
}

impl Clipboard for FakeClipboard {
    fn get(&self) -> String {
        match self.contents.try_borrow() {
            Ok(clipboard) => {
                clipboard.clone()
            }
            Err(e) => {
                error!("failed acquiring clipboard lock: {:?}", e);
                EMPTY_STRING
            }
        }
    }

    fn set(&self, contents: String) -> bool {
        match self.contents.try_borrow_mut() {
            Ok(mut cr) => {
                *cr = contents;
                true
            }
            Err(e) => {
                error!("failed acquiring clipboard lock: {:?}", e);
                false
            }
        }
    }
}