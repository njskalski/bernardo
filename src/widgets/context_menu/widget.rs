use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use log::{debug, error, warn};
use unicode_segmentation::UnicodeSegmentation;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::common_query::CommonQuery;
use crate::primitives::printable::Printable;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::tree_node::{ClosureFilter, TreeItFilter, TreeNode};
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::combined_widget::CombinedWidget;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WidgetActionParam, WID};
use crate::widgets::context_menu::msg::ContextMenuMsg;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;
use crate::{subwidget, unpack_unit_e};

pub const DEFAULT_SIZE: XY = XY::new(20, 10);
pub const CONTEXT_MENU_WIDGET_NAME: &'static str = "context_menu";

pub struct ContextMenuWidget<Key: Hash + Eq + Debug + Clone + 'static, Item: TreeNode<Key> + 'static> {
    id: WID,
    size: XY, //TODO never used
    config: ConfigRef,

    query_box: EditBoxWidget,
    tree_view: WithScroll<TreeViewWidget<Key, Item>>,

    layout_res: Option<LayoutResult<Self>>,

    on_close: Option<WidgetAction<ContextMenuWidget<Key, Item>>>,
}

impl<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> ContextMenuWidget<Key, Item> {
    pub fn new(providers: Providers, root_node: Item) -> Self {
        let query_box = EditBoxWidget::new()
            .with_size_policy(SizePolicy::MATCH_LAYOUTS_WIDTH)
            .with_on_change(Box::new(|editbox| ContextMenuMsg::UpdateQuery(editbox.get_text()).someboxed()));

        Self {
            id: get_new_widget_id(),
            size: DEFAULT_SIZE,
            config: providers.config().clone(),
            query_box,
            tree_view: WithScroll::new(
                ScrollDirection::Vertical,
                TreeViewWidget::new(root_node)
                    .with_size_policy(SizePolicy::MATCH_LAYOUT)
                    .with_filter_overrides_expanded()
                    .with_filter_policy(FilterPolicy::MatchNodeOrAncestors),
            ),
            layout_res: None,
            on_close: None,
        }
    }

    pub fn with_on_close(self, on_close: WidgetAction<ContextMenuWidget<Key, Item>>) -> Self {
        Self {
            on_close: Some(on_close),
            ..self
        }
    }

    pub fn set_on_close(&mut self, on_close_op: Option<WidgetAction<ContextMenuWidget<Key, Item>>>) {
        self.on_close = on_close_op;
    }

    pub fn with_on_hit(mut self, on_hit: WidgetAction<TreeViewWidget<Key, Item>>) -> Self {
        self.tree_view.internal_mut().set_on_hit_op(Some(on_hit));
        self
    }

    pub fn autoexpand_if_single_subtree(mut self) -> Self {
        if self.tree_view.internal().get_root_node().is_single_subtree() {
            self.tree_view.internal_mut().expand_root();
        }

        self
    }

    pub fn with_expanded_root(mut self) -> Self {
        self.tree_view.internal_mut().expand_root();
        self
    }

    pub fn expand_root(&mut self) {
        self.tree_view.internal_mut().expand_root();
    }

    pub fn set_on_shortcut_hit(&mut self, on_shortcut_hit: WidgetActionParam<TreeViewWidget<Key, Item>, Item>) {
        self.tree_view.internal_mut().set_on_shortcut_hit(on_shortcut_hit)
    }

    fn input_to_treeview(&self, input_event: &InputEvent) -> bool {
        match input_event {
            InputEvent::KeyInput(key) => match key.keycode {
                Keycode::ArrowUp => true,
                Keycode::ArrowDown => true,
                Keycode::ArrowLeft => true,
                Keycode::ArrowRight => true,
                Keycode::Enter => true,
                Keycode::PageUp => true,
                Keycode::PageDown => true,
                _ if self.tree_view.internal().are_shortcuts_enabled() => {
                    // let all_keycodes: Vec<_> = self.tree_view.internal().get_all_shortcuts().collect();
                    // error!("nananna {:?}", all_keycodes);

                    self.tree_view
                        .internal()
                        .get_all_shortcuts()
                        .find(|(_, _, bound_key)| *bound_key == *key)
                        .is_some()
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn tree_view(&self) -> &TreeViewWidget<Key, Item> {
        self.tree_view.internal()
    }

    pub fn tree_view_mut(&mut self) -> &mut TreeViewWidget<Key, Item> {
        self.tree_view.internal_mut()
    }
}

impl<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> Widget for ContextMenuWidget<Key, Item> {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        CONTEXT_MENU_WIDGET_NAME
    }

    fn typename(&self) -> &'static str {
        CONTEXT_MENU_WIDGET_NAME
    }

    fn prelayout(&mut self) {
        self.combined_prelayout()
    }

    fn full_size(&self) -> XY {
        self.tree_view.full_size() + XY::new(0, 1)
    }

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::MATCH_LAYOUT
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.combined_layout(screenspace);
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        match input_event {
            InputEvent::KeyInput(key) if key == self.config.keyboard_config.global.close_context_menu => ContextMenuMsg::Close.someboxed(),
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<ContextMenuMsg>();
        if our_msg.is_none() {
            debug!("expecetd ContextMenuMsg, got {:?}, passing through", msg);
            return Some(msg);
        }

        match our_msg.unwrap() {
            ContextMenuMsg::UpdateQuery(query) => {
                if query.is_empty() {
                    let tree_view = self.tree_view.internal_mut();
                    tree_view.set_filter_op(None, FilterPolicy::MatchNodeOrAncestors);
                    tree_view.set_highlighter(None);
                } else {
                    let query_fuzz = CommonQuery::Fuzzy(query.clone()); //TODO unnecessary copy
                    let query_copy = query.clone();

                    let tree_view = self.tree_view.internal_mut();
                    tree_view.set_filter_op(
                        Some(ClosureFilter::new(move |item: &Item| query_fuzz.matches(item.label().as_ref())).arc_box()),
                        FilterPolicy::MatchNodeOrAncestors,
                    );

                    tree_view.set_highlighter(Some(Box::new(move |label: &str| -> Vec<usize> {
                        let mut label_grapheme_it = label.graphemes(false).enumerate().peekable();
                        let mut query_grapheme_it = query_copy.graphemes().peekable();

                        let mut result: Vec<usize> = Vec::new();

                        while let (Some((label_idx, label_grapheme)), Some(query_grapheme)) =
                            (label_grapheme_it.peek(), query_grapheme_it.peek())
                        {
                            if **label_grapheme == **query_grapheme {
                                result.push(*label_idx);
                                let _ = query_grapheme_it.next();
                            }
                            let _ = label_grapheme_it.next();
                        }

                        if query_grapheme_it.peek().is_some() {
                            let non_displayed_characters = query_grapheme_it.collect::<Vec<_>>();
                            warn!(
                                "did not highlight entire query - filter desynchronized. Leftover characters: {:?}",
                                non_displayed_characters
                            );
                        }

                        result
                    })));
                }

                None
            }
            ContextMenuMsg::Close => {
                if let Some(on_close) = self.on_close.as_ref() {
                    on_close(self)
                } else {
                    warn!("received close message, but no on-miss is defined");
                    None
                }
            }
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let size = unpack_unit_e!(self.get_layout_res().map(|lr| lr.total_size), "render before layout",);

        #[cfg(test)]
        {
            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: CONTEXT_MENU_WIDGET_NAME.to_string(),
                rect: crate::primitives::rect::Rect::from_zero(size),
                focused,
            });
        }

        self.combined_render(theme, focused, output)
    }

    fn act_on(&mut self, input_event: InputEvent) -> (bool, Option<Box<dyn AnyMsg>>) {
        let mut act_result = if self.input_to_treeview(&input_event) {
            self.tree_view.act_on(input_event)
        } else {
            self.query_box.act_on(input_event)
        };

        if act_result.0 == false {
            // not consumed
            if let Some(msg_to_self) = self.on_input(input_event) {
                act_result = (true, Some(msg_to_self));
            }
        }

        if let Some(msg_to_myself) = act_result.1 {
            debug_assert!(act_result.0);

            (true, self.update(msg_to_myself))
        } else {
            act_result
        }
    }

    fn get_status_description(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed("context menu"))
    }
}

impl<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> CombinedWidget for ContextMenuWidget<Key, Item> {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Fixed(1), LeafLayout::new(subwidget!(Self.query_box)).boxed())
            .with(SplitRule::Proportional(1.0f32), LeafLayout::new(subwidget!(Self.tree_view)).boxed())
            .boxed()
    }

    fn save_layout_res(&mut self, result: LayoutResult<Self>) {
        self.layout_res = Some(result);
    }

    fn get_layout_res(&self) -> Option<&LayoutResult<Self>> {
        self.layout_res.as_ref()
    }

    fn get_subwidgets_for_input(&self) -> impl Iterator<Item = SubwidgetPointer<Self>> {
        [subwidget!(Self.tree_view), subwidget!(Self.query_box)].into_iter()
    }
}
