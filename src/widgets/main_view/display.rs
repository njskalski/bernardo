use std::borrow::Cow;

use crate::fs::path::SPath;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::Widget;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::main_view::main_view::MainView;
use crate::widgets::main_view::msg::MainViewMsg;

pub enum MainViewDisplay {
    Editor(EditorView),
    ResultsView(CodeResultsView),
}

impl MainViewDisplay {
    pub fn get_widget(&self) -> &dyn Widget {
        match self {
            MainViewDisplay::Editor(e) => e,
            MainViewDisplay::ResultsView(r) => r,
        }
    }

    pub fn get_widget_mut(&mut self) -> &mut dyn Widget {
        match self {
            MainViewDisplay::Editor(e) => e,
            MainViewDisplay::ResultsView(r) => r,
        }
    }
}

