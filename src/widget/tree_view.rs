use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::io::output::Output;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Formatter, Pointer};
use std::hash::Hash;
use std::ptr::write_bytes;

trait TreeViewNode<Key: Hash + Eq + Debug> {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn children(&self) -> Box<dyn std::iter::Iterator<Item = &dyn TreeViewNode<Key>> + '_>;
}

impl<Key: Hash + Eq + Debug> std::fmt::Debug for dyn TreeViewNode<Key> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeViewNode({:?})", self.id())
    }
}

fn tree_it<'a, Key: Hash + Eq + Debug>(
    root: &'a dyn TreeViewNode<Key>,
    expanded: &'a dyn Fn(&Key) -> bool,
) -> Box<dyn std::iter::Iterator<Item = &'a dyn TreeViewNode<Key>> + 'a> {
    if !expanded(root.id()) {
        Box::new(std::iter::once(root))
    } else {
        Box::new(
            std::iter::once(root).chain(
                root.children()
                    .flat_map(move |child| tree_it(child, expanded)),
            ),
        )
    }
}

#[derive(Debug)]
struct TreeView<Key: Hash + Eq + Debug> {
    id: WID,
    filter: String,
    filter_enabled: bool,
    root_node: Box<dyn TreeViewNode<Key>>,

    expanded: HashSet<Key>,
}

impl<Key: Hash + Eq + Debug> TreeView<Key> {
    pub fn new(root_node: Box<dyn TreeViewNode<Key>>) -> Self {
        Self {
            id: get_new_widget_id(),
            root_node,
            filter_enabled: false,
            filter: "".to_owned(),

            expanded: HashSet::new(),
        }
    }

    pub fn with_filter_enabled(self, enabled: bool) -> Self {
        TreeView {
            filter_enabled: enabled,
            ..self
        }
    }
}

impl<Key: Hash + Eq + Debug> Widget for TreeView<Key> {
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
    use crate::io::keys::Key;
    use crate::widget::tree_view::{tree_it, TreeViewNode};
    use std::collections::HashSet;

    struct StupidTree {
        id: usize,
        children: Vec<StupidTree>,
    }

    impl TreeViewNode<usize> for StupidTree {
        fn id(&self) -> &usize {
            &self.id
        }

        fn label(&self) -> String {
            format!("StupidTree {}", self.id)
        }

        fn children(&self) -> Box<dyn std::iter::Iterator<Item = &dyn TreeViewNode<usize>> + '_> {
            Box::new(self.children.iter().map(|f| f as &dyn TreeViewNode<usize>))
        }
    }

    #[test]
    fn tree_it_test_1() {
        let root = StupidTree {
            id: 0,
            children: vec![
                StupidTree {
                    id: 1,
                    children: vec![
                        StupidTree {
                            id: 10001,
                            children: vec![],
                        },
                        StupidTree {
                            id: 10002,
                            children: vec![],
                        },
                    ],
                },
                StupidTree {
                    id: 2,
                    children: vec![
                        StupidTree {
                            id: 20001,
                            children: vec![],
                        },
                        StupidTree {
                            id: 20002,
                            children: vec![],
                        },
                        StupidTree {
                            id: 20003,
                            children: vec![StupidTree {
                                id: 2000301,
                                children: vec![],
                            }],
                        },
                    ],
                },
            ],
        };

        let mut expanded: HashSet<usize> = HashSet::new();
        expanded.insert(0);
        expanded.insert(1);

        {
            let is_expanded = Box::new(|key: &usize| expanded.contains(key));
            let items: Vec<String> = tree_it(&root, &is_expanded)
                .map(|f| format!("{:?}", f.id()))
                .collect();
            let max_len = items.iter().fold(
                0,
                |acc, item| if acc > item.len() { acc } else { item.len() },
            );
            assert_eq!(items.len(), 5);
            assert_eq!(max_len, 5);
        }

        expanded.insert(2);

        {
            let is_expanded = Box::new(|key: &usize| expanded.contains(key));
            let items: Vec<String> = tree_it(&root, &is_expanded)
                .map(|f| format!("{:?}", f.id()))
                .collect();
            let max_len = items.iter().fold(
                0,
                |acc, item| if acc > item.len() { acc } else { item.len() },
            );
            assert_eq!(items.len(), 8);
            assert_eq!(max_len, 5);
        }

        expanded.insert(20003);

        {
            let is_expanded = Box::new(|key: &usize| expanded.contains(key));
            let items: Vec<String> = tree_it(&root, &is_expanded)
                .map(|f| format!("{:?}", f.id()))
                .collect();
            let max_len = items.iter().fold(
                0,
                |acc, item| if acc > item.len() { acc } else { item.len() },
            );
            assert_eq!(items.len(), 9);
            assert_eq!(max_len, 7);
        }
    }
}
