use crate::io::output::Metadata;
use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::treeview_interpreter::{TreeViewInterpreter, TreeViewInterpreterItem};
use crate::widgets::context_menu;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::tree_view::tree_view;

pub struct ContextMenuInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    tree_view: TreeViewInterpreter<'a>,
    editbox: EditWidgetInterpreter<'a>,
}

#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    pub label: String,
    pub highlighted: bool,
}

impl<'a> ContextMenuInterpreter<'a> {
    pub fn new(output: &'a MetaOutputFrame, meta: &'a Metadata) -> Self {
        debug_assert!(meta.typename == context_menu::widget::CONTEXT_MENU_WIDGET_NAME);

        let tree_view_meta: Vec<&Metadata> = output
            .get_meta_by_type(tree_view::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(tree_view_meta.len() == 1);
        let tree_view = TreeViewInterpreter::new(tree_view_meta[0], output);

        let editorbox_widget_meta: Vec<&Metadata> = output
            .get_meta_by_type(EditBoxWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(editorbox_widget_meta.len() == 1);
        let editbox = EditWidgetInterpreter::new(editorbox_widget_meta[0], output);

        ContextMenuInterpreter {
            meta,
            output,
            tree_view,
            editbox,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn tree_view(&self) -> &'a TreeViewInterpreter {
        &self.tree_view
    }

    pub fn editbox(&self) -> &'a EditWidgetInterpreter {
        &self.editbox
    }

    pub fn visible_items(&self) -> Vec<TreeViewInterpreterItem> {
        self.tree_view.items()
    }

    pub fn selected_option(&self) -> Option<String> {
        self.visible_items()
            .into_iter()
            .filter(|item| item.highlighted)
            .map(|item| item.label)
            .next()
    }

    pub fn meta(&self) -> &'a Metadata {
        self.meta
    }
}
