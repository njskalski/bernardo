use std::rc::Rc;
use std::sync::Arc;

pub enum BetterDerefStr<'a> {
    String(String),
    Rc(Rc<String>),
    Arc(Arc<String>),
    Str(&'a str),
}

impl<'a> BetterDerefStr<'a> {
    fn as_ref_str(&self) -> &str {
        match self {
            BetterDerefStr::String(s) => s.as_str(),
            BetterDerefStr::Rc(s) => s.as_str(),
            BetterDerefStr::Arc(s) => s.as_str(),
            BetterDerefStr::Str(s) => s,
        }
    }
}