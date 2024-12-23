use std::borrow::Cow;

use crate::widget::context_bar_item::ContextBarItem;
use crate::widget::widget::Widget;
use crate::widgets::context_menu::widget::ContextMenuWidget;

// First parameter determines depth on the Focus Path, 0 being the root.
// TODO we can have a collision of identifiers here
pub type MainContextMenuWidget = ContextMenuWidget<Cow<'static, str>, ContextBarItem>;

pub fn get_focus_path_w<'a>(root: &'a dyn Widget) -> Vec<&'a dyn Widget> {
    let mut result: Vec<&'a dyn Widget> = Vec::new();

    fn recursive_get_focus_path<'a>(root: &'a dyn Widget, result: &mut Vec<&'a dyn Widget>) {
        result.push(root);

        if let Some(subwidget) = root.get_focused() {
            recursive_get_focus_path(subwidget, result);
        }
    }

    recursive_get_focus_path(root, &mut result);

    result
}

pub fn aggregate_actions(widget: &dyn Widget) -> Vec<ContextBarItem> {
    let mut result: Vec<ContextBarItem> = Vec::new();

    for (idx, item) in get_focus_path_w(widget).iter().enumerate() {
        if let Some(mut action) = item.get_widget_actions() {
            action.set_depth_recursively(idx);
            result.push(action);
        }
    }

    result.reverse();

    result
}
