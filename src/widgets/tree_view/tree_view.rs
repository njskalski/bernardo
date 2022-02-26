use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use env_logger::filter::Filter;
use log::{debug, error, warn};
use termion::event::Key;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::arrow::Arrow;
use crate::primitives::helpers;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};
use crate::widgets::tree_view::tree_it::TreeIt;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub struct TreeViewWidget<Key: Hash + Eq + Debug + Clone, Item: TreeViewNode<Key>> {
    id: WID,
    root_node: Rc<Item>,
    filter_letters: Option<String>,
    expanded: HashSet<Key>,
    highlighted: usize,

    //events
    on_miss: Option<WidgetAction<TreeViewWidget<Key, Item>>>,
    on_highlighted_changed: Option<WidgetAction<TreeViewWidget<Key, Item>>>,
    on_flip_expand: Option<WidgetAction<TreeViewWidget<Key, Item>>>,
    // called on hitting "enter" over a selection.
    on_select_highlighted: Option<WidgetAction<TreeViewWidget<Key, Item>>>,
}

#[derive(Debug)]
enum TreeViewMsg {
    Arrow(Arrow),
    HitEnter,
}

impl AnyMsg for TreeViewMsg {}

/*
Warranties:
- (TODO double check) Highlighted always exists.
 */
impl<Key: Hash + Eq + Debug + Clone, Item: TreeViewNode<Key>> TreeViewWidget<Key, Item> {
    pub fn new(root_node: Rc<Item>) -> Self {
        Self {
            id: get_new_widget_id(),
            root_node,
            filter_letters: None,
            expanded: HashSet::new(),
            highlighted: 0,
            on_miss: None,
            on_highlighted_changed: None,
            on_flip_expand: None,
            on_select_highlighted: None,
        }
    }

    pub fn with_filter_letters(self, filter_letters: String) -> Self {
        TreeViewWidget {
            filter_letters: Some(filter_letters),
            ..self
        }
    }

    pub fn set_filter_letters(&mut self, filter_letters: Option<String>) {
        self.filter_letters = filter_letters;
    }

    pub fn is_expanded(&self, key: &Key) -> bool {
        self.expanded.contains(key)
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

    pub fn with_on_flip_expand(self, on_flip_expand: WidgetAction<TreeViewWidget<Key, Item>>) -> Self {
        Self {
            on_flip_expand: Some(on_flip_expand),
            ..self
        }
    }

    pub fn with_on_highlighted_changed(self, on_highlighted_changed: WidgetAction<TreeViewWidget<Key, Item>>) -> Self {
        Self {
            on_highlighted_changed: Some(on_highlighted_changed),
            ..self
        }
    }

    fn event_highlighted_changed(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_highlighted_changed.map(|f| f(self)).flatten()
    }

    pub fn with_on_select_hightlighted(self, on_select_highlighted: WidgetAction<TreeViewWidget<Key, Item>>) -> Self {
        Self {
            on_select_highlighted: Some(on_select_highlighted),
            ..self
        }
    }

    fn event_select_highlighted(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_select_highlighted.map(|f| f(self)).flatten()
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

    pub fn items(&self) -> TreeIt<Key, Item> {
        TreeIt::new(&self.root_node, &self.expanded)
    }

    pub fn get_highlighted(&self) -> (u16, Item) {
        self.items().nth(self.highlighted).unwrap() //TODO
    }
}

impl<K: Hash + Eq + Debug + Clone, I: TreeViewNode<K>> Widget for TreeViewWidget<K, I> {
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

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        let from_items = self.size_from_items();
        let mut res = sc.hint().size;

        if from_items.x > res.x && sc.x().is_none() {
            res.x = from_items.x;
        }

        if from_items.y > res.y && sc.y().is_none() {
            res.y = from_items.y;
        }

        res
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("tree_view.on_input {:?}", input_event);

        return match input_event {
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::ArrowUp => Some(Box::new(TreeViewMsg::Arrow(Arrow::Up))),
                    Keycode::ArrowDown => Some(Box::new(TreeViewMsg::Arrow(Arrow::Down))),
                    Keycode::Enter => { Some(Box::new(TreeViewMsg::HitEnter)) },
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
            },
            TreeViewMsg::HitEnter => {
                let node = {
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
                    highlighted_node
                };

                if node.is_leaf() {
                    self.event_select_highlighted()
                } else {
                    self.flip_expanded(node.id());
                    self.event_flip_expand()
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let primary_style = theme.default_text(focused);
        helpers::fill_output(primary_style.background, output);
        let cursor_style = theme.cursor().maybe_half(focused);

        for (idx, (depth, node)) in self.items().enumerate()
            // skipping lines that cannot be visible, because they are before hint()
            .skip(output.size_constraint().hint().upper_left().y as usize) {

            // skipping lines that cannot be visible, because larger than the hint()
            if idx >= output.size_constraint().hint().lower_right().y as usize {
                break;
            }

            // TODO this I think can be skipped
            match output.size_constraint().y() {
                Some(y) => if idx >= y as usize {
                    debug!("idx {}, output.size().y {}", idx, output.size_constraint());
                    break;
                }
                None => {}
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

            let mut x_offset: usize = 0;
            for g in text.graphemes(true).into_iter() {
                let desired_pos_x: usize = depth as usize * 2 + x_offset;
                if desired_pos_x > u16::MAX as usize {
                    error!("skipping drawing beyond x = u16::MAX");
                    break;
                }

                let x = desired_pos_x as u16;
                if x >= output.size_constraint().hint().lower_right().x {
                    break;
                }

                // This is fine, because idx is proved to be within output constraints, which by definition are u16.
                let y = idx as u16;

                output.print_at(
                    XY::new(x, y),
                    style,
                    g,
                );

                x_offset += g.width();
            }
        }
    }

    fn anchor(&self) -> XY {
        //TODO add x corresponding to depth
        XY::new(0, self.highlighted as u16) //TODO unsafe cast
    }
}
