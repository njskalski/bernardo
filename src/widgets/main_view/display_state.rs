use log::debug;
use crate::experiments::focus_group::{FocusGroup, FocusUpdate};
use crate::layout::layout::WidgetIdRect;
use crate::widget::widget::WID;

// TODO add multi column
#[derive(Debug, Eq, PartialEq)]
pub enum Focus {
    Tree,
    Editor,
}

#[derive(Debug)]
pub struct MainViewDisplayState {
    pub focus: Focus,
    pub curr_editor_idx: Option<usize>,
    pub wirs: Vec<WidgetIdRect>,
}

impl Default for MainViewDisplayState {
    fn default() -> Self {
        MainViewDisplayState {
            focus: Focus::Tree,
            curr_editor_idx: None,
            wirs: Vec::default(),
        }
    }
}

impl MainViewDisplayState {
    pub fn update_focus(&mut self, fu: FocusUpdate) -> bool {
        match fu {
            FocusUpdate::Left if self.focus == Focus::Editor => {
                self.focus = Focus::Tree;
                true
            }
            FocusUpdate::Right if self.focus == Focus::Tree => {
                self.focus = Focus::Editor;
                true
            }
            _ => false
        }
    }

    pub fn will_accept_update_focus(&self, fu: FocusUpdate) -> bool {
        match fu {
            FocusUpdate::Left if self.focus == Focus::Editor => true,
            FocusUpdate::Right if self.focus == Focus::Tree => true,
            _ => false
        }
    }
}
