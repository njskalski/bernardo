use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter;

use log::{debug, error, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::keys;
use crate::io::keys::{Key, Keycode};
use crate::io::output::Output;
use crate::primitives::arrow::Arrow;
use crate::primitives::helpers;
use crate::primitives::tree::tree_it::eager_iterator;
use crate::primitives::tree::tree_node::{TreeItFilter, TreeNode};
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WidgetActionParam, WID};
use crate::{unpack, unpack_or_e};

pub const TYPENAME: &str = "tree_view";

// expectation is that these are sorted
pub type LabelHighlighter = Box<dyn Fn(&str) -> Vec<usize>>;

// Keys are unique

pub struct TreeViewWidget<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> {
    id: WID,
    root_node: Item,
    expanded: HashSet<Key>,
    // TODO rethink that
    // at this point, highlighted can move (nodes can disappear if the filter throws them away with delay)
    highlighted: usize,

    last_size: Option<Screenspace>,

    //events
    on_miss: Option<WidgetAction<Self>>,
    on_highlighted_changed: Option<WidgetAction<Self>>,
    on_flip_expand: Option<WidgetAction<Self>>,
    // called on hitting "enter" over a selection.
    on_hit: Option<WidgetAction<Self>>,

    on_keyboard_shortcut_hit: Option<WidgetActionParam<Self, Item>>,

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

    size_policy: SizePolicy,

    // if set to true, all nodes which lead to non-empty subtrees will appear in view, even if not expanded.
    filter_overrides_expanded: bool,
}

#[derive(Debug)]
enum TreeViewMsg {
    Arrow(Arrow),
    HitEnter,
    ShortcutHit(keys::Key),
}

impl AnyMsg for TreeViewMsg {}

/*
Warranties:
- (TODO double check) Highlighted always exists.
- TODO change highlighted to Rc<Key>, because now with lazy loading and filtering, items can
    disappear...
 */
impl<Key: Hash + Eq + Debug + Clone + 'static, Item: TreeNode<Key> + 'static> TreeViewWidget<Key, Item> {
    pub fn new(root_node: Item) -> Self {
        Self {
            id: get_new_widget_id(),
            root_node,
            expanded: HashSet::new(),
            highlighted: 0,
            last_size: None,
            on_miss: None,
            on_highlighted_changed: None,
            on_flip_expand: None,
            on_hit: None,
            on_keyboard_shortcut_hit: None,
            highlighter_op: None,
            filter_op: None,
            filter_depth_op: None,
            size_policy: SizePolicy::MATCH_LAYOUT,
            filter_overrides_expanded: false,
        }
    }

    pub fn with_highlighter(self, highlighter: LabelHighlighter) -> Self {
        Self {
            highlighter_op: Some(highlighter),
            ..self
        }
    }

    pub fn with_filter_overrides_expanded(self) -> Self {
        Self {
            filter_overrides_expanded: true,
            ..self
        }
    }

    pub fn with_size_policy(self, size_policy: SizePolicy) -> Self {
        Self { size_policy, ..self }
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

    pub fn expand_root(&mut self) {
        self.expanded.insert(self.root_node.id().clone());
    }

    pub fn set_selected(&mut self, k: &Key) -> bool {
        for (pos, (_, item)) in self.items().enumerate() {
            if item.id() == k {
                self.highlighted = pos;
                return true;
            }
        }

        error!("failed to find item with key {:?}", k);

        false
    }

    fn size_from_items(&self) -> XY {
        let mut size = XY::ONE;

        for item in self.items() {
            size = XY::new(
                // depth * 2 + 2 + label_length. The +2 comes from the fact, that even at 0 depth, we add a triangle AND a space before the
                // label.
                size.x.max(item.0 as u16 * 2 + 2 + item.1.label().width() as u16), // TODO fight overflow here.
                size.y + 1,
            );
        }

        if size == XY::ONE {
            warn!("size == ONE. Empty item provider?");
        }

        size
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

    pub fn with_on_shortcut_hit(self, on_shortcut_hit: WidgetActionParam<Self, Item>) -> Self {
        Self {
            on_keyboard_shortcut_hit: Some(on_shortcut_hit),
            ..self
        }
    }

    pub fn set_on_shortcut_hit(&mut self, on_shortcut_hit: WidgetActionParam<Self, Item>) {
        self.on_keyboard_shortcut_hit = Some(on_shortcut_hit);
    }

    fn event_highlighted_changed(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_highlighted_changed.as_ref().and_then(|f: &WidgetAction<Self>| f(self))
    }

    pub fn with_on_hit(self, on_hit: WidgetAction<TreeViewWidget<Key, Item>>) -> Self {
        Self {
            on_hit: Some(on_hit),
            ..self
        }
    }

    pub fn set_on_hit_op(&mut self, on_hit_op: Option<WidgetAction<TreeViewWidget<Key, Item>>>) {
        self.on_hit = on_hit_op;
    }

    fn event_hit(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_hit.as_ref().and_then(|f: &WidgetAction<Self>| f(self))
    }

    fn event_miss(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_miss.as_ref().and_then(|f: &WidgetAction<Self>| f(self))
    }

    fn event_flip_expand(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_flip_expand.as_ref().and_then(|f: &WidgetAction<Self>| f(self))
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

    pub fn items(&self) -> impl Iterator<Item = (u16, Item)> {
        // TreeIt::new(&self.root_node, Some(&self.expanded), self.filter_op.as_ref(), self.filter_depth_op)
        if self.filter_overrides_expanded && self.filter_op.is_some() {
            eager_iterator(&self.root_node, None, self.filter_op.as_ref())
        } else {
            eager_iterator(&self.root_node, Some(&self.expanded), self.filter_op.as_ref())
        }
    }

    pub fn get_highlighted(&self) -> (u16, Item) {
        self.items().nth(self.highlighted).clone().unwrap() //TODO
    }

    pub fn get_root_node(&self) -> &Item {
        &self.root_node
    }

    // Does NOT check for presence of Key in the tree
    pub fn set_expanded(&mut self, key: Key, expanded: bool) {
        if expanded {
            self.expanded.insert(key);
        } else {
            self.expanded.remove(&key);
        }
    }

    pub fn expand_all_internal_nodes(&mut self) {
        fn expand<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>>(expanded: &mut HashSet<Key>, node: &Item) {
            if !node.is_leaf() {
                expanded.insert(node.id().clone());

                for child in node.child_iter() {
                    expand(expanded, &child);
                }
            }
        }

        expand(&mut self.expanded, &self.root_node);
    }

    // Returns true if Key was found AND the node was expanded.
    // Returns false if node was NOT expanded OR THE KEY IS ABSENT.
    pub fn get_expanded(&self, key: &Key) -> bool {
        self.expanded.contains(key)
    }

    // Idx, depth, item
    // TODO I think there is a bug there
    pub fn get_visible_items(&self) -> Box<dyn Iterator<Item = (usize, (u16, Item))> + 'static> {
        let screenspace = unpack_or_e!(
            self.last_size,
            Box::new(iter::empty()),
            "can't decide visible items without last_size"
        );
        let visible_rect = screenspace.visible_rect();

        Box::new(
            self.items()
                .enumerate()
                // skipping lines that cannot be visible, because they are before hint()
                .skip(visible_rect.upper_left().y as usize)
                .take(visible_rect.size.y as usize),
        )
    }

    pub fn get_all_shortcuts(&self) -> Box<dyn Iterator<Item = (usize, Key, keys::Key)> + 'static> {
        Box::new(
            self.items()
                .enumerate()
                .filter(|(_, (_, item))| item.keyboard_shortcut().is_some())
                .map(|(idx, (_, item))| (idx, item.id().clone(), item.keyboard_shortcut().unwrap())),
        )
    }

    // these 3 methods below are unused as of now, I am not sure I want to use them. If I do,
    // I need to figure out definition of page-up and page-down.
    // fn can_select(&self, key: &Key) -> bool {
    //     true
    // }
    //
    // fn get_prev_highlight(&self) -> Option<usize> {
    //     if self.highlighted == 0 {
    //         return None;
    //     }
    //
    //     let mut it = self.items().enumerate().take(self.highlighted - 1);
    //     let mut last_idx: Option<usize> = None;
    //
    //     while let Some((idx, (_, item))) = it.next() {
    //         if self.can_select(item.id()) {
    //             last_idx = Some(idx);
    //         }
    //     }
    //
    //     last_idx
    // }
    //
    // fn get_next_highlight(&self) -> Option<usize> {
    //     let mut it = self.items().enumerate().skip(self.highlighted);
    //
    //     while let Some((idx, (_, item))) = it.next() {
    //         if self.can_select(item.id()) {
    //             return Some(idx);
    //         }
    //     }
    //
    //     None
    // }
}

impl<K: Hash + Eq + Debug + Clone + 'static, I: TreeNode<K> + 'static> Widget for TreeViewWidget<K, I> {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        TYPENAME
    }

    fn typename(&self) -> &'static str {
        TYPENAME
    }

    fn full_size(&self) -> XY {
        self.size_from_items()
    }

    fn size_policy(&self) -> SizePolicy {
        self.size_policy
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.last_size = Some(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        // debug!("tree_view.on_input {:?}", input_event);

        match input_event {
            InputEvent::KeyInput(key) => match key.keycode {
                Keycode::ArrowUp => Some(Box::new(TreeViewMsg::Arrow(Arrow::Up))),
                Keycode::ArrowDown => Some(Box::new(TreeViewMsg::Arrow(Arrow::Down))),
                Keycode::Enter => Some(Box::new(TreeViewMsg::HitEnter)),
                _ => {
                    if let Some(action_trigger) = self.on_keyboard_shortcut_hit.as_ref() {
                        for item in self.items() {
                            if let Some(shortcut) = item.1.keyboard_shortcut() {
                                if key == shortcut {
                                    return TreeViewMsg::ShortcutHit(shortcut).someboxed();
                                }
                            }
                        }
                    }

                    None
                }
            },
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
                    let highlighted_pair = self.items().nth(self.highlighted);

                    if highlighted_pair.is_none() {
                        warn!("TreeViewWidget #{} highlighted non-existent node {}!", self.id(), self.highlighted);
                        return None;
                    }
                    let (_, highlighted_node) = highlighted_pair.unwrap();
                    highlighted_node
                };

                if node.is_leaf() {
                    self.event_hit()
                } else {
                    self.flip_expanded(node.id());
                    self.event_flip_expand()
                }
            }
            TreeViewMsg::ShortcutHit(key) => {
                if let Some(action_trigger) = self.on_keyboard_shortcut_hit.as_ref() {
                    for item in self.items() {
                        if let Some(shortcut) = item.1.keyboard_shortcut() {
                            if *key == shortcut {
                                return action_trigger(self, item.1);
                            }
                        }
                    }
                }
                error!("expected a valid key shortcut for {:?}, found none", key);
                None
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            let size = crate::unpack_unit_e!(self.last_size, "render before layout",);

            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: crate::primitives::rect::Rect::from_zero(size.output_size()),
                focused,
            });
        }

        let visible_rect = output.visible_rect();

        let primary_style = theme.default_text(focused);
        helpers::fill_output(primary_style.background, output);
        let cursor_style = theme.highlighted(focused);

        for (item_idx, (depth, node)) in self
            .items()
            .enumerate()
            // skipping lines that cannot be visible, because they are before hint()
            .skip(visible_rect.upper_left().y as usize)
        {
            // skipping lines that cannot be visible, because larger than the hint()
            if item_idx >= visible_rect.lower_right().y as usize {
                break;
            }

            if item_idx >= output.visible_rect().lower_right().y as usize {
                debug!(
                    "idx {}, output.visible_rect().y {}",
                    item_idx,
                    output.visible_rect().lower_right().y
                );
                break;
            }

            let style = if item_idx == self.highlighted {
                cursor_style
            } else {
                primary_style
            };

            let prefix = if node.is_leaf() {
                " "
            } else if self.expanded.contains(node.id()) {
                "▶"
            } else {
                "▼"
            };

            let text = format!("{} {}", prefix, node.label());
            let highlighted: Vec<usize> = self.highlighter_op.as_ref().map(|h| h(&text)).unwrap_or_default();
            let mut highlighted_iter = highlighted.into_iter().peekable();

            // This is fine, because idx is proved to be within output constraints, which by definition are u16.
            let y = item_idx as u16;

            let mut x_offset: usize = 0;
            for (grapheme_idx, g) in text.graphemes(true).enumerate() {
                let desired_pos_x: usize = depth as usize * 2 + x_offset;
                if desired_pos_x > u16::MAX as usize {
                    error!("skipping drawing beyond x = u16::MAX");
                    break;
                }

                let x = desired_pos_x as u16;
                if x >= visible_rect.lower_right().x {
                    break;
                }

                let mut local_style = style;

                //highlighted_idx < highlighted.len() && highlighted[highlighted_idx] == grapheme_idx
                if let Some(highlighted_idx) = highlighted_iter.peek() {
                    if *highlighted_idx == grapheme_idx {
                        local_style = local_style.with_background(theme.ui.focused_highlighted.background);
                        let _ = highlighted_iter.next();
                    }
                }

                output.print_at(XY::new(x, y), local_style, g);

                x_offset += g.width();
            }

            // drawing label
            if let Some(key) = node.keyboard_shortcut() {
                x_offset += 2;

                let label = key.to_string();

                for g in label.graphemes(true) {
                    let desired_pos_x: usize = depth as usize * 2 + x_offset;
                    if desired_pos_x > u16::MAX as usize {
                        error!("skipping drawing beyond x = u16::MAX");
                        break;
                    }

                    let x = desired_pos_x as u16;
                    if x >= visible_rect.lower_right().x {
                        break;
                    }

                    let style = theme.editor_label_warning();

                    output.print_at(XY::new(x, y), style, g);
                    x_offset += g.width();
                }
            }
        }
    }

    fn kite(&self) -> XY {
        //TODO add x corresponding to depth
        XY::new(0, self.highlighted as u16) //TODO unsafe cast
    }

    fn get_status_description(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed("tree view"))
    }
}
