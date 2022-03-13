use std::rc::Rc;
use std::sync::Arc;

pub trait DerefStr {
    fn as_ref_str(&self) -> &str;
}

impl DerefStr for &str {
    fn as_ref_str(&self) -> &str {
        self
    }
}


impl DerefStr for Rc<String> {
    fn as_ref_str(&self) -> &str {
        self.as_str()
    }
}

impl DerefStr for Arc<String> {
    fn as_ref_str(&self) -> &str {
        self.as_str()
    }
}

impl DerefStr for String {
    fn as_ref_str(&self) -> &str {
        self.as_str()
    }
}

