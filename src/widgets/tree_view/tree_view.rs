use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

use log::{debug, error, warn};
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
use crate::widgets::tree_view::tree_view_node::{TreeItFilter, TreeViewNode};

// expectation is that these are sorted
pub type LabelHighlighter = fn(&str) -> Vec<usize>;

// Keys are unique

pub struct TreeViewWidget<Key: Hash + Eq + Debug + Clone, Item: TreeViewNode<Key>> {
    id: WID,
    root_node: Item,
    expanded: HashSet<Key>,
    // TODO rethink that
    // at this point, highlighted can move (nodes can disappear if the filter throws them away with delay)
    highlighted: usize,

    //events
    on_miss: Option<WidgetAction<TreeViewWidget<Key, Item>>>,
    on_highlighted_changed: Option<WidgetAction<TreeViewWidget<Key, Item>>>,
    on_flip_expand: Option<WidgetAction<TreeViewWidget<Key, Item>>>,
    // called on hitting "enter" over a selection.
    on_select_highlighted: Option<WidgetAction<TreeViewWidget<Key, Item>>>,

    // This will highlight letters given their indices. Use to do "fuzzy search" in tree.
    highlighter_op: Option<LabelHighlighter>,
    // This is a filter that will be applied to decide which items to show or not.
    filter_op: Option<TreeItFilter<Item>>,
    // This tells whether to follow to dive down looking for filter matching nodes, even if
    // parent node fails filter. The idea is: "I'm looking for files with infix X, but will
    // display their non-matching parents too".
    // filter_depth = None means "don't dive"
    // filter_depth = Some(x) means "look for items down to x levels down".
    filter_depth_op: Option<usize>,
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
- TODO change highlighted to Rc<Key>, because now with lazy loading and filtering, items can
    disappear...
 */
impl<Key: Hash + Eq + Debug + Clone, Item: TreeViewNode<Key>> TreeViewWidget<Key, Item> {
    pub fn new(root_node: Item) -> Self {
        Self {
            id: get_new_widget_id(),
            root_node,
            expanded: HashSet::new(),
            highlighted: 0,
            on_miss: None,
            on_highlighted_changed: None,
            on_flip_expand: None,
            on_select_highlighted: None,
            highlighter_op: None,
            filter_op: None,
            filter_depth_op: None,
        }
    }

    pub fn with_highlighter(self, highlighter: LabelHighlighter) -> Self {
        Self {
            highlighter_op: Some(highlighter),
            ..self
        }
    }

    pub fn set_highlighter(&mut self, highlighter_op: Option<LabelHighlighter>) {
        self.highlighter_op = highlighter_op;
    }

    pub fn with_filter(self, filter: TreeItFilter<Item>, filter_depth_op: Option<usize>) -> Self {
        Self {
            filter_op: Some(filter),
            filter_depth_op,
            ..self
        }
    }

    pub fn set_filter_op(&mut self, filter_op: Option<TreeItFilter<Item>>, filter_depth_op: Option<usize>) {
        self.filter_op = filter_op;
        self.filter_depth_op = filter_depth_op;
    }

    pub fn is_expanded(&self, key: &Key) -> bool {
        self.expanded.contains(key)
    }

    pub fn expanded_mut(&mut self) -> &mut HashSet<Key> {
        &mut self.expanded
    }

    pub fn expanded(&self) -> &HashSet<Key> {
        &self.expanded
    }

    pub fn set_selected(&mut self, k: &Key) -> bool {
        let mut pos = 0 as usize;
        for (_, item) in self.items() {
            if item.id() == k {
                self.highlighted = pos;
                return true;
            }
            pos += 1;
        }

        error!("failed to find item with key {:?}", k);

        false
    }

    fn size_from_items(&self) -> XY {
        self.items().fold(ZERO, |old_size, item| {
            // debug!("adding item (depth {}) {:?}", item.0, item.1);

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
        TreeIt::new(&self.root_node, &self.expanded, &self.filter_op, self.filter_depth_op)
    }

    pub fn get_highlighted(&self) -> (u16, Item) {
        self.items().nth(self.highlighted).clone().unwrap() //TODO
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
                    Keycode::Enter => { Some(Box::new(TreeViewMsg::HitEnter)) }
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
        let cursor_style = theme.highlighted(focused);

        for (item_idx, (depth, node)) in self.items().enumerate()
            // skipping lines that cannot be visible, because they are before hint()
            .skip(output.size_constraint().hint().upper_left().y as usize) {

            // skipping lines that cannot be visible, because larger than the hint()
            if item_idx >= output.size_constraint().hint().lower_right().y as usize {
                break;
            }

            // TODO this I think can be skipped
            match output.size_constraint().y() {
                Some(y) => if item_idx >= y as usize {
                    debug!("idx {}, output.size().y {}", item_idx, output.size_constraint());
                    break;
                }
                None => {}
            }

            let style = if item_idx == self.highlighted {
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
            let higlighted: Vec<usize> = self.highlighter_op.map(
                |h| h(&text)
            ).unwrap_or(vec![]);
            let highlighted_idx: usize = 0;

            let mut x_offset: usize = 0;
            for (grapheme_idx, g) in text.graphemes(true).into_iter().enumerate() {
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
                let y = item_idx as u16;

                let mut local_style = style;

                if highlighted_idx < higlighted.len() {
                    if higlighted[highlighted_idx] == grapheme_idx {
                        local_style = local_style.with_background(theme.ui.focused_highlighted.background);
                    }
                }

                output.print_at(
                    XY::new(x, y),
                    local_style,
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
