use std::rc::Rc;
use std::sync::Arc;

pub enum BetterDerefStr<'a> {
    String(String),
    Rc(Rc<String>),
    Arc(Arc<String>),
    StaticStr(&'static str),
    Str(&'a str),
}

impl<'a> BetterDerefStr<'a> {
    pub fn as_ref_str(&self) -> &str {
        match self {
            BetterDerefStr::String(s) => s.as_str(),
            BetterDerefStr::Rc(s) => s.as_str(),
            BetterDerefStr::Arc(s) => s.as_str(),
            BetterDerefStr::StaticStr(s) => s,
            BetterDerefStr::Str(s) => s,
        }
    }
}

impl<'a> From<&'static str> for BetterDerefStr<'a> {
    fn from(s: &'static str) -> Self {
        BetterDerefStr::StaticStr(s)
    }
}