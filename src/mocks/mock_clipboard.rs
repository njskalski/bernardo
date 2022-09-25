use std::ops::Deref;
use std::sync::RwLock;

use log::error;

use crate::experiments::clipboard::Clipboard;

#[derive(Debug, Default)]
pub struct MockClipboard {
    pub contents: RwLock<String>,
}

impl Clipboard for MockClipboard {
    fn get(&self) -> String {
        match self.contents.read() {
            Ok(c) => {
                c.deref().to_string()
            }
            Err(e) => {
                error!("failed acquiring lock: {:?}", e);
                String::new()
            }
        }
    }

    fn set(&self, s: String) -> bool {
        match self.contents.write() {
            Ok(mut lock) => {
                *lock = s;
                true
            }
            Err(e) => {
                error!("failed acquiring lock: {:?}", e);
                false
            }
        }
    }
}