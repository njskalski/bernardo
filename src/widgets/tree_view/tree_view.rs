use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use log::{debug, warn};
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::arrow::Arrow;
use crate::primitives::helpers;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};
use crate::widgets::tree_view::tree_it::TreeIt;
use crate::widgets::tree_view::tree_view_node::{ChildRc, TreeViewNode};

pub struct TreeViewWidget<Key: Hash + Eq + Debug + Clone> {
    id: WID,
    filter: String,
    filter_enabled: bool,
    root_node: Rc<dyn TreeViewNode<Key>>,

    expanded: HashSet<Key>,
    highlighted: usize,

    //events
    on_miss: Option<WidgetAction<TreeViewWidget<Key>>>,
    on_highlighted_changed: Option<WidgetAction<TreeViewWidget<Key>>>,
    on_flip_expand: Option<WidgetAction<TreeViewWidget<Key>>>,
}

#[derive(Debug)]
enum TreeViewMsg {
    Arrow(Arrow),
    FlipExpansion,
}

impl AnyMsg for TreeViewMsg {}

/*
Warranties:
- (TODO double check) Highlighted always exists.
 */
impl<Key: Hash + Eq + Debug + Clone> TreeViewWidget<Key> {
    pub fn new(root_node: Rc<dyn TreeViewNode<Key>>) -> Self {
        Self {
            id: get_new_widget_id(),
            root_node,
            filter_enabled: false,
            filter: "".to_owned(),
            expanded: HashSet::new(),
            highlighted: 0,
            on_miss: None,
            on_highlighted_changed: None,
            on_flip_expand: None,
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
        self.items().fold(ZERO, |old_size, item| {
            XY::new(
                // depth * 2 + 1 + label_length
                old_size
                    .x
                    .max(item.0 * 2 + 1 + item.1.label().width() as u16), // TODO fight overflow here.
                old_size.y + 1,
            )
        })
    }

    pub fn with_on_flip_expand(self, on_flip_expand: WidgetAction<TreeViewWidget<Key>>) -> Self {
        Self {
            on_flip_expand: Some(on_flip_expand),
            ..self
        }
    }

    pub fn with_on_highlighted_changed(self, on_highlighted_changed: WidgetAction<TreeViewWidget<Key>>) -> Self {
        Self {
            on_highlighted_changed: Some(on_highlighted_changed),
            ..self
        }
    }

    fn event_highlighted_changed(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_highlighted_changed.map(|f| f(self)).flatten()
    }

    fn event_miss(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_miss.map(|f| f(self)).flatten()
    }

    fn event_flip_expand(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_flip_expand.map(|f| f(self)).flatten()
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

    pub fn items(&self) -> TreeIt<Key> {
        TreeIt::new(self.root_node.clone(), &self.expanded)
    }

    pub fn get_highlighted(&self) -> (u16, ChildRc<Key>) {
        self.items().nth(self.highlighted).unwrap()
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

    fn layout(&mut self, max_size: XY) -> XY {
        // let mut from_items = self.size_from_items();
        //
        // debug!("would like {} limits to {}", from_items, max_size);
        // from_items = from_items.cut(max_size);
        // debug!("results {}", from_items);
        //
        // from_items
        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("tree_view.on_input {:?}", input_event);

        return match input_event {
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::Char('a') => Some(Box::new(TreeViewMsg::FlipExpansion)),
                    Keycode::ArrowUp => Some(Box::new(TreeViewMsg::Arrow(Arrow::Up))),
                    Keycode::ArrowDown => Some(Box::new(TreeViewMsg::Arrow(Arrow::Down))),
                    Keycode::ArrowLeft => Some(Box::new(TreeViewMsg::Arrow(Arrow::Left))),
                    Keycode::ArrowRight => Some(Box::new(TreeViewMsg::Arrow(Arrow::Right))),
                    Keycode::Enter => Some(Box::new(TreeViewMsg::FlipExpansion)),
                    _ => None,
                }
            }
            _ => None,
        };
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
                    if self.highlighted < self.items().count() - 1 {
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
                    let highlighted_pair = self.items().skip(self.highlighted).next();

                    if highlighted_pair.is_none() {
                        warn!(
                            "TreeViewWidget #{} highlighted non-existent node {}!",
                            self.id(),
                            self.highlighted
                        );
                        return None;
                    }
                    let (_, highlighted_node) = highlighted_pair.unwrap();
                    (highlighted_node.id().clone(), highlighted_node.is_leaf()) // TODO can we avoid the clone?
                };

                if is_leaf {
                    self.event_miss()
                } else {
                    self.flip_expanded(&id);
                    self.event_flip_expand()
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let primary_style = theme.default_text(focused);
        helpers::fill_background(primary_style.background, output);
        let cursor_style = theme.cursor().maybe_half(focused);

        for (idx, (depth, node)) in self.items().enumerate() {
            if idx >= output.size().y as usize {
                debug!("idx {}, output.size().y {}", idx, output.size());
                break;
            }


            let style = if idx == self.highlighted {
                cursor_style
            } else {
                primary_style
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

    fn anchor(&self) -> XY {
        //TODO add x corresponding to depth
        XY::new( 0, self.highlighted as u16) //TODO unsafe cast
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::io::keys::Keycode;
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
