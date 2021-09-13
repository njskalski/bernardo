use crate::widget::widget::{Widget, WID, get_new_widget_id};
use crate::primitives::xy::XY;
use crate::io::input_event::InputEvent;
use crate::widget::any_msg::AnyMsg;
use crate::io::output::Output;
use std::collections::{HashSet, HashMap};
use std::hash::Hash;
use std::borrow::Borrow;
use crate::io::keys::Key;

trait TreeViewNode<Key : Hash + Eq> {
    fn id(&self) -> Key;
    fn label(&self) -> String;
    fn children(&self) -> Box<Iterator<Item=Box<&dyn TreeViewNode<Key>>>>;
}

fn some<'a, Key : Hash + Eq>(root : Box<&'a dyn TreeViewNode<Key>>, expanded : &HashSet<Key>) -> Box<Iterator<Item=Box<&'a dyn TreeViewNode<Key>>> + 'a>{ // eee makarena!
    if expanded.contains(&root.id()) {
        Box::new(std::iter::once(root))

        
    } else {
        Box::new(std::iter::once(root))
    }
}

struct TreeView<Key : Hash + Eq>  {
    id : WID,
    filter : String,
    filter_enabled : bool,
    root_node : Box<dyn TreeViewNode<Key>>,

    expanded : HashSet<Key>,
}

impl <Key : Hash + Eq> TreeView<Key> {
    pub fn new(root_node : Box<dyn TreeViewNode<Key>>) -> Self {
        Self {
            id : get_new_widget_id(),
            root_node,
            filter_enabled : false,
            filter : "".to_owned(),

            expanded : HashSet::new()
        }
    }

    pub fn with_filter_enabled(self, enabled : bool) -> Self {
        TreeView {
            filter_enabled : enabled,
            ..self
        }
    }
}

impl <Key : Hash + Eq> Widget for TreeView<Key> {
    fn id(&self) -> WID {
        self.id
    }

    fn min_size(&self) -> XY {
        XY::new(3, 10)
    }

    fn size(&self, max_size: XY) -> XY {
        todo!()

        // max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn get_focused(&self) -> &dyn Widget {
        todo!()
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        todo!()
    }

    fn render(&self, focused: bool, output: &mut Output) {
        todo!()
    }
}

#[cfg(test)]
mod tests {

}

