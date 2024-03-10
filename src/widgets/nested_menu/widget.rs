use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use log::{error, warn};

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::{Key, Keycode};
use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::tree::tree_node::TreeNode;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::button::ButtonWidgetMsg;
use crate::widgets::nested_menu;
use crate::widgets::nested_menu::msg::Msg;

/*
This describes a simple context menu.
For first version, options remain fixed (no adding/deleting)

There is no optimisation here whatsoever. First let it work, second write the test, then optimise.
 */

pub struct NestedMenuWidget<Key: Hash + Eq + Debug, Item: TreeNode<Key>> {
    wid: WID,

    max_size: XY,

    layout_size: Option<XY>,
    // key, label
    selected_nodes: Vec<(Key, String)>,

    // we count selected row only within selected node. Selected node is determined based on
    // "root" item and "selected nodes".
    selected_row_idx: u16,

    root: Item,
    _phantom: PhantomData<Key>,
}

impl<Key: Hash + Eq + Debug, Item: TreeNode<Key>> NestedMenuWidget<Key, Item> {

    pub const EXPANSION_WIDTH : u16 = 2;
    pub fn new(root_node: Item, max_size: XY) -> Self {
        NestedMenuWidget {
            wid: get_new_widget_id(),
            max_size,
            layout_size: None,
            selected_nodes: Default::default(),
            selected_row_idx: 0,
            root: root_node,
            _phantom: Default::default(),
        }
    }

    fn get_subtree(&self) -> Option<Item> {
        let mut item = self.root.clone();

        for exp in self.selected_nodes.iter() {
            // TODO unwritten assumption that there is no duplicate keys
            if let Some(subtree) = item.child_iter().find(|item| item.id() == &exp.0) {
                item = subtree;
                continue;
            } else {
                error!("key {:?} not found", exp);
                return None;
            }
        }

        Some(item)
    }
}

impl<Key: Hash + Eq + Debug + 'static, Item: TreeNode<Key> + 'static> Widget for NestedMenuWidget<Key, Item> {
    fn id(&self) -> WID {
        self.wid
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        "NestedMenuWidget"
    }

    fn typename(&self) -> &'static str {
        "NestedMenuWidget"
    }

    fn full_size(&self) -> XY {
        self.max_size
    }

    fn layout(&mut self, screenspace: Screenspace) {
        let actual_size = screenspace.output_size();
        self.layout_size = Some(actual_size);
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            KeyInput(key_event) => match key_event.keycode {
                Keycode::Enter => Some(Box::new(nested_menu::msg::Msg::Hit)),
                _ => None,
            },
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<nested_menu::msg::Msg>();
        if our_msg.is_none() {
            warn!("expecetd nested_menu Msg, got {:?}", msg);
            return None;
        }

        match our_msg.unwrap() {
            Msg::Hit => None,
            _ => None,
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let item_expanded: u16 = 0;
        let mut expanded_items: u16 = 0;

        let base = theme.default_text(focused);
        let cursor = theme.highlighted(true);
        let selected_nodes = TextStyle::new(cursor.foreground, base.background, Effect::None);

        // TODO overflow
        for y in 0..std::cmp::min(self.selected_nodes.len() as u16, output.size().y) {
            let begin_pos = XY::new(y * Self::EXPANSION_WIDTH, y);
            output.print_at(begin_pos, selected_nodes, self.selected_nodes[y as usize].1.as_str())
        }

        let left_height : u16 = if output.size().y as usize > self.selected_nodes.len() {
            output.size().y - (self.selected_nodes.len() as u16)
        } else {
            0
        };

        let x_offset: u16 = Self::EXPANSION_WIDTH * (self.selected_nodes.len() as u16);
        let y_offset: u16 = self.selected_nodes.len() as u16; // TODO overflow

        if let Some(subtree) = self.get_subtree() {
            let mut idx : u16 = 0;
            for item in subtree.child_iter() {
                if idx >= left_height {
                    break;
                }

                let highlighted = self.selected_row_idx == idx;
                let style = if highlighted {
                    theme.highlighted(focused)
                } else {
                    theme.default_text(focused)
                };

                if item.is_leaf() == false {
                    output.print_at(XY::new(x_offset, y_offset + idx),
                                    style,
                                    "> ");
                } else {
                    output.print_at(XY::new(x_offset, y_offset + idx),
                                    style,
                                    "  ");
                }

                output.print_at(
                    XY::new(x_offset + Self::EXPANSION_WIDTH, y_offset + idx),
                    style,
                    item.label().as_ref()
                );


                idx += 1;
            }
        }
    }
}
