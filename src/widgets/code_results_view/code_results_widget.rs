use std::rc::Rc;

use crate::config::config::ConfigRef;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::layout::split_layout::SplitRule;
use crate::primitives::scroll::ScrollDirection;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::big_list::big_list_widget::BigList;
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::text_widget::TextWidget;
use crate::widgets::with_scroll::WithScroll;

pub struct CodeResultsView {
    wid: WID,

    finished_loading: bool,
    label: TextWidget,
    item_list: WithScroll<BigList<EditorWidget>>,

    //providers
    data_provider: Box<dyn CodeResultsProvider>,
    config: ConfigRef,
    tree_sitter: Rc<TreeSitterWrapper>,
    fsf: FsfRef,
    clipboard: ClipboardRef,
}

impl CodeResultsView {
    pub fn new(
        config: ConfigRef,
        tree_sitter: Rc<TreeSitterWrapper>,
        fsf: FsfRef,
        clipboard: ClipboardRef,
        label: String,
        data_provider: Box<dyn CodeResultsProvider>,
    ) -> Self {
        Self {
            wid: get_new_widget_id(),
            finished_loading: false,
            label: TextWidget::new(Box::new(label)),
            item_list: WithScroll::new(ScrollDirection::Vertical,
                                       BigList::new(vec![]),
            ),
            data_provider,
            config,
            tree_sitter,
            fsf,
            clipboard,
        }
    }
}
