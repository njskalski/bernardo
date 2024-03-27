use std::fmt::Debug;
use std::hash::Hash;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::tree::tree_node::TreeNode;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::combined_widget::CombinedWidget;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::nested_menu::widget::NESTED_MENU_TYPENAME;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;
use crate::{subwidget, unpack_unit_e};

pub const DEFAULT_SIZE: XY = XY::new(20, 10);
pub const CONTEXT_MENU_WIDGET_NAME: &'static str = "context_menu";

pub struct ContextMenuWidget<Key: Hash + Eq + Debug + Clone + 'static, Item: TreeNode<Key> + 'static> {
    id: WID,
    size: XY,
    query_box: EditBoxWidget,
    tree_view: WithScroll<TreeViewWidget<Key, Item>>,

    layout_res: Option<LayoutResult<Self>>,
}

impl<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> ContextMenuWidget<Key, Item> {
    pub fn new(providers: Providers, root_node: Item) -> Self {
        Self {
            id: get_new_widget_id(),
            size: DEFAULT_SIZE,
            query_box: EditBoxWidget::new().with_size_policy(SizePolicy::MATCH_LAYOUTS_WIDTH),
            tree_view: WithScroll::new(
                ScrollDirection::Vertical,
                TreeViewWidget::new(root_node).with_size_policy(SizePolicy::MATCH_LAYOUT),
            ),
            layout_res: None,
        }
    }

    fn input_to_treeview(input_event: &InputEvent) -> bool {
        match input_event {
            InputEvent::KeyInput(key) => match key.keycode {
                Keycode::ArrowUp => true,
                Keycode::ArrowDown => true,
                Keycode::ArrowLeft => true,
                Keycode::ArrowRight => true,
                Keycode::PageUp => true,
                Keycode::PageDown => true,
                _ => false,
            },
            _ => false,
        }
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

    fn layout(&mut self, screenspace: Screenspace) {
        self.combined_layout(screenspace);
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
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

    // fn kite(&self) -> XY {
    //     todo!()
    // }

    fn act_on(&mut self, input_event: InputEvent) -> (bool, Option<Box<dyn AnyMsg>>) {
        let act_result = if Self::input_to_treeview(&input_event) {
            self.tree_view.act_on(input_event)
        } else {
            self.query_box.act_on(input_event)
        };

        if let Some(msg_to_myself) = act_result.1 {
            debug_assert!(act_result.0);

            (true, self.update(msg_to_myself))
        } else {
            act_result
        }
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
}
