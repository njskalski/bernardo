use std::fmt::Debug;

use crate::cursor::cursor::Cursor;
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::w7e::navcomp_provider::CompletionAction;
use crate::widget::any_msg::AnyMsg;

#[derive(Clone, Debug)]
pub enum EditorWidgetMsg {
    EditMsg(CommonEditMsg),

    ToCursorDropMode,
    ToEditMode,

    DropCursorFlip { cursor: Cursor },
    // not sure if this can't be simplified, why separate message?
    DropCursorMove { cem: CommonEditMsg },

    OpenContextMenu,
    ContextMenuClose,

    RequestCompletions,
    HoverClose,
    CompletionWidgetSelected(CompletionAction),

    RequestContextBar,

    Reformat,
    GoToDefinition,
    ShowUsages,
}

impl AnyMsg for EditorWidgetMsg {}
