use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

/*
This is a helper enum to enable returning immutable ref str wrappers from methods without introducing generics.

I actually need a help to make it better, because I refuse to spend too much time here. Basically I need an enum
that can be either &str, owned String or Cow for printing purposes.
 */
pub enum BetterDerefStr<'a> {
    String(String),
    Rc(Rc<String>),
    Arc(Arc<String>),
    StaticStr(&'static str),
    Str(&'a str),
    Cow(Cow<'a, &'a str>),
}

impl<'a> BetterDerefStr<'a> {
    pub fn as_ref_str(&self) -> &str {
        match self {
            BetterDerefStr::String(s) => s.as_str(),
            BetterDerefStr::Rc(s) => s.as_str(),
            BetterDerefStr::Arc(s) => s.as_str(),
            BetterDerefStr::StaticStr(s) => s,
            BetterDerefStr::Str(s) => s,
            BetterDerefStr::Cow(cow) => cow.as_ref(),
        }
    }
}

impl<'a> From<&'static str> for BetterDerefStr<'a> {
    fn from(s: &'static str) -> Self {
        BetterDerefStr::StaticStr(s)
    }
}
