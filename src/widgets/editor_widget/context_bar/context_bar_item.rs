use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

use crate::widgets::list_widget::list_widget_item::ListWidgetItem;

// I think I want the "context bar" to be "cascading", enabling a "conversational like" interface.
// Because some of commands will yield "options" to choose from, say "usages" will show up multiple
// options. Here's how I imagine that.

/*
    box drawing symbols are for clarity only, I will use colors to distinguish between layers and
    save screen space

    somesymbol
        ┌──────────────────── ...
        │ go to definition
        │ [find usages]
        │ some other option
        │
       ...

     hit enter
    somesymbol
        ┌──────────────────── ...
        │ find usages ↴
        │ ┌────────────────── ...
        │ │ usage 1 (file X)
        │ │ usage 2 (file Y)
        │ │
       .....

    escape goes back

    findings:
    1) Would be great to anchor the bar in the middle of symbol (say middle of word).
       At the same time, depending on language, different symbols can be even queried. For instance,
       one cannot overload operators in pure C (so there is no good definition for == operator)
       but in say Scala, pretty much everything like "-++-" can become an operator and "jump to
       definition" of it makes perfect sense.
       So "what even is the symbol under my cursor" is a call to NavComp provider.

 */

#[derive(Debug, Clone)]
pub struct ContextBarItem {
    title: String,
}

impl ListWidgetItem for ContextBarItem {
    fn get_column_name(idx: usize) -> &'static str {
        "name"
    }

    fn get_min_column_width(idx: usize) -> u16 {
        match idx {
            0 => 10,
            _ => 0,
        }
    }

    fn len_columns() -> usize {
        1
    }

    fn get(&self, idx: usize) -> Option<Cow<'_, str>> {
        match idx {
            0 => Some(Cow::Borrowed(&self.title)),
            _ => None,
        }
    }
}