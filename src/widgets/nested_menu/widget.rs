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
use crate::primitives::arrow::Arrow;
use crate::primitives::has_invariant::HasInvariant;
use crate::primitives::printable::Printable;
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

pub const NESTED_MENU_TYPENAME: &'static str = "nested_menu";
pub const NESTED_MENU_FOLDER_CLOSED: char = '>';
pub const NESTED_MENU_FOLDER_OPENED: char = 'v';
pub const NESTED_MENU_FOLDER_WIDHT: u16 = 2;

pub struct NestedMenuWidget<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> {
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

pub fn get_highlighted_style(theme: &Theme, focused: bool) -> TextStyle {
    theme.highlighted(focused)
}

pub fn get_expanded_style(theme: &Theme, focused: bool) -> TextStyle {
    let highlighted = theme.highlighted(focused);
    let default = theme.default_text(focused);

    TextStyle::new(highlighted.foreground, default.background, Effect::None)
}

pub fn get_default_style(theme: &Theme, focused: bool) -> TextStyle {
    theme.default_text(focused)
}

impl<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> NestedMenuWidget<Key, Item> {
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

    fn get_selected_item(&self) -> Option<Item> {
        self.get_subtree()
            .map(|subtree| subtree.child_iter().skip(self.selected_row_idx as usize).next())
            .flatten()
    }

    fn get_subtree_height(&self) -> u16 {
        // TODO overflow
        self.get_subtree().map(|item| item.child_iter().count()).unwrap_or(0) as u16
    }
}

impl<Key: Hash + Eq + Debug + Clone + 'static, Item: TreeNode<Key> + 'static> Widget for NestedMenuWidget<Key, Item> {
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
                Keycode::Enter => Some(Box::new(Msg::Hit)),
                Keycode::ArrowUp => {
                    if self.selected_row_idx > 0 {
                        Some(Box::new(Msg::Arrow(Arrow::Up)))
                    } else {
                        None
                    }
                }
                Keycode::ArrowDown => {
                    if self.get_subtree_height() > self.selected_row_idx + 1 {
                        Some(Box::new(Msg::Arrow(Arrow::Down)))
                    } else {
                        None
                    }
                }
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

        return match our_msg.unwrap() {
            Msg::Hit => {
                if let Some(item) = self.get_selected_item() {
                    if item.is_leaf() {
                        // TODO action
                        None
                    } else {
                        self.selected_nodes.push((item.id().clone(), item.label().to_string()));
                        None
                    }
                } else {
                    warn!("did not get selected item!");
                    None
                }
            }
            _ => None,
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            let size = crate::unpack_unit_e!(self.layout_size, "render before layout",);

            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: NESTED_MENU_TYPENAME.to_string(),
                rect: crate::primitives::rect::Rect::from_zero(size),
                focused,
            });
        }

        let item_expanded: u16 = 0;
        let mut expanded_items: u16 = 0;

        let base_style = get_default_style(theme, focused);
        let cursor_style = get_highlighted_style(theme, focused);
        let expanded_nodes_style = get_expanded_style(theme, focused);

        // TODO overflow
        for y in 0..std::cmp::min(self.selected_nodes.len() as u16, output.size().y) {
            let begin_pos = XY::new(y * NESTED_MENU_FOLDER_WIDHT, y);
            output.print_at(begin_pos, expanded_nodes_style, self.selected_nodes[y as usize].1.as_str())
        }

        let left_height: u16 = if output.size().y as usize > self.selected_nodes.len() {
            output.size().y - (self.selected_nodes.len() as u16)
        } else {
            0
        };

        let x_offset: u16 = NESTED_MENU_FOLDER_WIDHT * (self.selected_nodes.len() as u16);
        let y_offset: u16 = self.selected_nodes.len() as u16; // TODO overflow

        if let Some(subtree) = self.get_subtree() {
            let mut idx: u16 = 0;
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
                    output.print_at(XY::new(x_offset, y_offset + idx), style, "> ");
                } else {
                    output.print_at(XY::new(x_offset, y_offset + idx), style, "  ");
                }

                output.print_at(
                    XY::new(x_offset + 1 * NESTED_MENU_FOLDER_WIDHT, y_offset + idx),
                    style,
                    item.label().as_ref(),
                );

                idx += 1;
            }
        }
    }
}
