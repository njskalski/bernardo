use std::borrow::Cow;

use crate::fs::path::SPath;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::Widget;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::fuzzy_search::item_provider;
use crate::widgets::fuzzy_search::item_provider::Item;
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

impl item_provider::Item for (usize, &MainViewDisplay) {
    fn display_name(&self) -> Cow<str> {
        match self.1 {
            MainViewDisplay::Editor(editor) => {
                match editor.buffer_state().get_path() {
                    Some(path) => {
                        path.label()
                    }
                    None => {
                        format!("unnamed buffer #{}", self.0)
                    }
                }
            }
            MainViewDisplay::ResultsView(results) => {
                results.get_text()
            }
        }
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        MainViewMsg::FuzzyBuffersHit { pos: self.0 }.boxed()
    }
}

impl item_provider::ItemsProvider for &Vec<MainViewDisplay> {
    fn context_name(&self) -> &str {
        "displays"
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        Box::new(self.iter().enumerate())
    }
}