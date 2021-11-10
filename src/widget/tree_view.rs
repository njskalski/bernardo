use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Formatter, Pointer};
use std::hash::Hash;
use std::ptr::write_bytes;

use log::warn;
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::io::output::Output;
use crate::io::style::{TextStyle_WhiteOnBlack, TextStyle_WhiteOnBrightYellow};
use crate::primitives::arrow::Arrow;
use crate::primitives::xy::{XY, Zero};
use crate::widget::any_msg::AnyMsg;
use crate::widget::edit_box::EditBoxWidget;
use crate::widget::tree_view_node::TreeViewNode;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};

// fn tree_it_2<'a, Key: Hash + Eq + Debug + Clone>(
//     root: &'a dyn TreeViewNode<Key>,
//     expanded: &'a HashSet<Key>,
// ) -> Vec<(u16, &'a dyn TreeViewNode<Key>)> {
//     let is_expanded = move |key : &Key| {expanded.contains(key)};
//
//     let x : Vec<_> = tree_it_rec(root, &is_expanded, 0).collect();
//     x.clone()
// }

fn tree_it<'a, Key: Hash + Eq + Debug + Clone>(
    root: &'a dyn TreeViewNode<Key>,
    expanded: &'a HashSet<Key>,
) -> Box<dyn std::iter::Iterator<Item = (u16, &'a dyn TreeViewNode<Key>)> + 'a> {
    tree_it_rec(root, expanded, 0)
}

fn tree_it_rec<'a, Key: Hash + Eq + Debug + Clone>(
    root: &'a dyn TreeViewNode<Key>,
    expanded: &'a HashSet<Key>,
    depth: u16,
) -> Box<dyn std::iter::Iterator<Item = (u16, &'a dyn TreeViewNode<Key>)> + 'a> {
    if !expanded.contains(root.id()) {
        Box::new(std::iter::once((depth, root)))
    } else {
        Box::new(
            std::iter::once((depth, root)).chain(
                root.children()
                    .flat_map(move |child| tree_it_rec(child, expanded, depth + 1)),
            ),
        )
    }
}

pub struct TreeViewWidget<Key: Hash + Eq + Debug + Clone> {
    id: WID,
    filter: String,
    filter_enabled: bool,
    root_node: Box<dyn TreeViewNode<Key>>,

    expanded: HashSet<Key>,
    highlighted: usize,

    items_num: usize,

    //events
    on_miss: Option<WidgetAction<TreeViewWidget<Key>>>,
    on_highlighted_changed: Option<WidgetAction<TreeViewWidget<Key>>>,
}

#[derive(Debug)]
enum TreeViewMsg {
    Arrow(Arrow),
    FlipExpansion,
}

impl AnyMsg for TreeViewMsg {}

impl<Key: Hash + Eq + Debug + Clone> TreeViewWidget<Key> {
    pub fn new(root_node: Box<dyn TreeViewNode<Key>>) -> Self {
        Self {
            id: get_new_widget_id(),
            root_node,
            filter_enabled: false,
            filter: "".to_owned(),
            expanded: HashSet::new(),
            highlighted: 0,
            items_num: 1,
            on_miss: None,
            on_highlighted_changed: None,
        }
    }

    pub fn with_filter_enabled(self, enabled: bool) -> Self {
        TreeViewWidget {
            filter_enabled: enabled,
            ..self
        }
    }

    fn get_is_expanded(&self) -> Box<dyn Fn(&Key) -> bool + '_> {
        Box::new(move |key: &Key| self.expanded.contains(key))
    }

    fn size_from_items(&self) -> XY {
        self.items().fold(Zero, |old_size, item| {
            XY::new(
                // depth * 2 + 1 + label_length
                old_size
                    .x
                    .max(item.0 * 2 + 1 + item.1.label().width() as u16), // TODO fight overflow here.
                old_size.y + 1,
            )
        })
    }

    fn event_highlighted_changed(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_highlighted_changed.is_some() {
            self.on_highlighted_changed.unwrap()(self)
        } else {
            None
        }
    }

    fn event_miss(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_miss.is_some() {
            self.on_miss.unwrap()(self)
        } else {
            None
        }
    }

    fn get_highlighted_node(&self) -> Option<&dyn TreeViewNode<Key>> {
        self.items().skip(self.highlighted).next().map(|p| p.1)
    }

    // returns new value
    fn flip_expanded(&mut self, id_to_flip: &Key) -> bool {
        if self.expanded.contains(id_to_flip) {
            self.expanded.remove(id_to_flip);
            false
        } else {
            self.expanded.insert(id_to_flip.clone());
            true
        }
    }

    fn items(&self) -> Box<dyn Iterator<Item = (u16, &dyn TreeViewNode<Key>)> + '_> {
        tree_it(&*self.root_node, &self.expanded)
    }
}

impl<K: Hash + Eq + Debug + Clone> Widget for TreeViewWidget<K> {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "TreeView"
    }

    fn min_size(&self) -> XY {
        let mut from_items = self.size_from_items();

        if from_items.x < 5 {
            from_items.x = 5;
        };
        if from_items.y < 1 {
            from_items.y = 1;
        };

        from_items
    }

    fn layout(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        match input_event {
            InputEvent::KeyInput(key) => {
                match key {
                    Key::Letter('a') => Some(Box::new(TreeViewMsg::FlipExpansion)),
                    Key::ArrowUp => Some(Box::new(TreeViewMsg::Arrow(Arrow::Up))),
                    Key::ArrowDown => Some(Box::new(TreeViewMsg::Arrow(Arrow::Down))),
                    Key::ArrowLeft => Some(Box::new(TreeViewMsg::Arrow(Arrow::Left))),
                    Key::ArrowRight => Some(Box::new(TreeViewMsg::Arrow(Arrow::Right))),
                    Key::Enter => Some(Box::new(TreeViewMsg::FlipExpansion)),
                    // Key::Space => {}
                    // Key::Backspace => {}
                    // Key::Home => {}
                    // Key::End => {}
                    // Key::PageUp => {}
                    // Key::PageDown => {}
                    // Key::Delete => {}
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<TreeViewMsg>();
        if our_msg.is_none() {
            warn!("expecetd TreeViewMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            TreeViewMsg::Arrow(arrow) => match arrow {
                Arrow::Up => {
                    if self.highlighted > 0 {
                        self.highlighted -= 1;
                        self.event_highlighted_changed()
                    } else {
                        self.event_miss()
                    }
                }
                Arrow::Down => {
                    if self.highlighted < self.items_num - 1 {
                        self.highlighted += 1;
                        self.event_highlighted_changed()
                    } else {
                        self.event_miss()
                    }
                }
                _ => None,
                // Arrow::Left => {}
                // Arrow::Right => {}
            },
            TreeViewMsg::FlipExpansion => {
                let (id, is_leaf) = {
                    let highlighted_node_op = self.get_highlighted_node();

                    if highlighted_node_op.is_none() {
                        warn!(
                            "TreeViewWidget #{} highlighted non-existent node {}!",
                            self.id(),
                            self.highlighted
                        );
                        return None;
                    }
                    let highlighted_node = highlighted_node_op.unwrap();
                    (highlighted_node.id().clone(), highlighted_node.is_leaf()) // TODO can we avoid the clone?
                };

                if is_leaf {
                    self.event_miss()
                } else {
                    self.flip_expanded(&id);

                    self.items_num = self.items().count();

                    None
                }
            }
        };
    }

    fn get_focused(&self) -> &dyn Widget {
        self
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn render(&self, focused: bool, output: &mut Output) {
        for (idx, (depth, node)) in self.items().enumerate() {
            let style = if idx == self.highlighted {
                TextStyle_WhiteOnBrightYellow
            } else {
                TextStyle_WhiteOnBlack
            };

            let prefix = if node.is_leaf() {
                " "
            } else {
                if self.expanded.contains(node.id()) {
                    "▶"
                } else {
                    "▼"
                }
            };

            let text = format!("{} {}", prefix, node.label());

            output.print_at(
                XY::new(depth * 2, idx as u16), // TODO idx in u16
                style,
                text.as_str(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::io::keys::Key;
    use crate::widget::stupid_tree::{get_stupid_tree, StupidTree};
    use crate::widget::tree_view::{tree_it, TreeViewNode};
    use crate::widget::widget::get_new_widget_id;

    #[test]
    fn tree_it_test_1() {
        let root = get_stupid_tree();

        let mut expanded: HashSet<usize> = HashSet::new();
        expanded.insert(0);
        expanded.insert(1);

        let try_out = |expanded_ref: &HashSet<usize>| {
            let items: Vec<(u16, String)> = tree_it(&root, expanded_ref)
                .map(|(d, f)| (d, format!("{:?}", f.id())))
                .collect();
            let max_len = items.iter().fold(
                0,
                |acc, (_, item)| if acc > item.len() { acc } else { item.len() },
            );
            (items, max_len)
        };

        {
            let (items, max_len) = try_out(&expanded);
            assert_eq!(items.len(), 5);
            assert_eq!(max_len, 5);
        }

        expanded.insert(2);

        {
            let (items, max_len) = try_out(&expanded);
            assert_eq!(items.len(), 8);
            assert_eq!(max_len, 5);
        }

        expanded.insert(20003);

        {
            let (items, max_len) = try_out(&expanded);
            assert_eq!(items.len(), 9);
            assert_eq!(max_len, 7);
        }
    }
}
