use crate::widget::widget::Widget;

pub fn get_focus_path(root: &dyn Widget) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    fn internal(result: &mut Vec<String>, current_node: &dyn Widget) {
        if let Some(desc) = current_node.get_status_description() {
            result.push(desc.to_string());
        }

        if let Some(child) = current_node.get_focused() {
            internal(result, child);
        }
    }

    internal(&mut result, root);

    result
}
