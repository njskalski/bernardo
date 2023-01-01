use std::borrow::Cow;
use std::fmt::Debug;

use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widgets::editor_widget::msg::EditorWidgetMsg;
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

pub type Action = fn() -> Box<dyn AnyMsg>;

/*
TODO
 I am not sure how this struct should look like inside, I just know how I want it to look in UI.
 Therefore I delay this design until I have working tests.
 */
#[derive(Debug, Clone)]
pub struct ContextBarItem {
    title: Cow<'static, str>,
    action: Action,
}

impl ContextBarItem {
    pub const GO_TO_DEFINITION: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("go to definition"),
        action: || EditorWidgetMsg::GoToDefinition.boxed(),
    };
    pub const REFORMAT_FILE: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("reformat file"),
        action: || EditorWidgetMsg::Reformat.boxed(),
    };
    pub const SHOW_USAGES: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("show usages"),
        action: || EditorWidgetMsg::ShowUsages.boxed(),
    };
    // TODO add reformat selection

    pub fn msg(&self) -> Box<dyn AnyMsg> {
        (self.action)()
    }
}


impl ListWidgetItem for ContextBarItem {
    fn get_column_name(_idx: usize) -> &'static str {
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