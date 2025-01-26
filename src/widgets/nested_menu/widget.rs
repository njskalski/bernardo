use std::fmt::Debug;
use std::hash::Hash;

use log::{error, warn};
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::arrow::VerticalArrow;
use crate::primitives::tree::tree_node::TreeNode;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::nested_menu;
use crate::widgets::nested_menu::msg::Msg;

/*
This describes a simple context menu.
For first version, options remain fixed (no adding/deleting).

There is no optimisation here whatsoever. First let it work, second write the test, then optimise.

Let's describe how it should look like:

v ExpandedSubtree1
  v ExpandedSubSubtree1
      SelectableItem1
      SelectableItem2
  > NotExpandedSubtree2
    SelectableItem3

Actions:
- hitting enter on expanded tree collapses it
- hitting enter on collapsed tree expands it
- hitting enter on selectable item causes a message to be emmited
- arrows navigate, they can jump between expanded subtrees

- query. Query works as in...

fuck this is just another tree_view.
 */

pub const NESTED_MENU_TYPENAME: &'static str = "nested_menu";
pub const NESTED_MENU_FOLDER_CLOSED: &'static str = ">";
pub const NESTED_MENU_FOLDER_OPENED: &'static str = "v";
pub const NESTED_MENU_FOLDER_WIDHT: u16 = 2;

pub struct NestedMenuWidget<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> {
    wid: WID,
    mapper: Option<Box<dyn Fn(&Item) -> Option<Box<dyn AnyMsg>> + 'static>>,

    max_size: XY,
    layout_size: Option<XY>,
    // key, label
    selected_nodes: Vec<(Key, String)>,

    // we count selected row only within selected node. Selected node is determined based on
    // "root" item and "selected nodes".
    selected_row_idx: u16,

    root: Item,

    query: BufferSharedRef,
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
    pub fn new(providers: Providers, root_node: Item, max_size: XY) -> Self {
        let query_buffer = BufferState::simplified_single_line().into_bsr();

        NestedMenuWidget {
            wid: get_new_widget_id(),
            mapper: None,
            max_size,
            layout_size: None,
            selected_nodes: Default::default(),
            selected_row_idx: 0,
            root: root_node,
            query: BufferState::simplified_single_line().into_bsr(),
        }
    }

    pub fn with_mapper<M: Fn(&Item) -> Option<Box<dyn AnyMsg>> + 'static>(self, m: M) -> Self {
        Self {
            mapper: Some(Box::new(m)),
            ..self
        }
    }

    fn get_subtree(&self) -> Option<Item> {
        let mut item = self.root.clone();

        for exp in self.selected_nodes.iter() {
            // TODO unwritten assumption that there is no duplicate keys
            let mut next: Option<Item> = None;
            if let Some(subtree) = item.child_iter().find(|item| item.id() == &exp.0) {
                next = Some(subtree);
            } else {
                error!("key {:?} not found", exp);
                return None;
            };

            if let Some(next) = next {
                item = next;
            } else {
                return None; // never reached
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
            KeyInput(key) if key == Keycode::Enter.to_key() => Some(Box::new(Msg::Hit)),
            KeyInput(key) if key == Keycode::ArrowUp.to_key() => {
                if self.selected_row_idx > 0 {
                    Msg::Arrow(VerticalArrow::Up).someboxed()
                } else {
                    None
                }
            }
            KeyInput(key) if key == Keycode::ArrowDown.to_key() => {
                if self.get_subtree_height() > self.selected_row_idx + 1 {
                    Msg::Arrow(VerticalArrow::Down).someboxed()
                } else {
                    None
                }
            }
            KeyInput(key) if key == Keycode::ArrowRight.to_key() => {
                if self.get_selected_item().map(|item| item.is_leaf() == false).unwrap_or(false) {
                    Msg::Hit.someboxed()
                } else {
                    None
                }
            }
            KeyInput(key) if key == Keycode::ArrowLeft.to_key() => {
                if self.selected_nodes.is_empty() == false {
                    Msg::UnwrapOneLevel.someboxed()
                } else {
                    None
                }
            }
            // KeyInput(key) if key_to_edit_msg(key).is_some() => {
            //     let msg = key_to_edit_msg(key).unwrap();
            //
            //     let ignore: bool = match msg {
            //         CommonEditMsg::Char(_) => false,
            //         CommonEditMsg::Block(_) => true,
            //         CommonEditMsg::CursorUp { .. } => true,
            //         CommonEditMsg::CursorDown { .. } => true,
            //         CommonEditMsg::CursorLeft { .. } => true,
            //         CommonEditMsg::CursorRight { .. } => true,
            //         CommonEditMsg::Backspace => false,
            //         CommonEditMsg::LineBegin { .. } => true,
            //         CommonEditMsg::LineEnd { .. } => true,
            //         CommonEditMsg::WordBegin { .. } => true,
            //         CommonEditMsg::WordEnd { .. } => true,
            //         CommonEditMsg::PageUp { .. } => true,
            //         CommonEditMsg::PageDown { .. } => true,
            //         CommonEditMsg::Delete => true,
            //         CommonEditMsg::Copy => true,
            //         CommonEditMsg::Paste => true,
            //         CommonEditMsg::Undo => true,
            //         CommonEditMsg::Redo => true,
            //         CommonEditMsg::DeleteBlock { .. } => true,
            //         CommonEditMsg::InsertBlock { .. } => true,
            //         CommonEditMsg::SubstituteBlock { .. } => true,
            //         CommonEditMsg::Tab => true,
            //         CommonEditMsg::ShiftTab => true,
            //     };
            //
            //     if !ignore {
            //         Msg::QueryEdit(msg).someboxed()
            //     } else {
            //         None
            //     }
            // }
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
                        if let Some(mapper) = &self.mapper {
                            (*mapper)(&item)
                        } else {
                            error!("selection mapper not set");
                            None
                        }
                    } else {
                        self.selected_nodes.push((item.id().clone(), item.label().to_string()));
                        self.selected_row_idx = 0;
                        None
                    }
                } else {
                    warn!("did not get selected item!");
                    None
                }
            }
            Msg::Arrow(arrow) => match arrow {
                VerticalArrow::Up => {
                    if self.selected_row_idx > 0 {
                        self.selected_row_idx -= 1;
                    } else {
                        error!("can't arrow up");
                    }
                    None
                }
                VerticalArrow::Down => {
                    if self.selected_row_idx + 1 < self.get_subtree_height() {
                        self.selected_row_idx += 1;
                    } else {
                        error!("can't arrow down");
                    }
                    None
                }
            },
            Msg::UnwrapOneLevel => {
                if self.selected_nodes.is_empty() {
                    error!("can't unwrap one level");
                    None
                } else {
                    let last_item = self.selected_nodes.pop().unwrap().0;
                    let idx = if let Some(subtree) = self.get_subtree() {
                        let mut idx: u16 = 0;
                        for item in subtree.child_iter() {
                            if *item.id() == last_item {
                                break;
                            }
                            idx += 1;
                        }
                        idx
                    } else {
                        error!("can't figure out selected index from ");
                        0
                    };

                    self.selected_row_idx = idx;
                    None
                }
            }
            _ => None,
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let size = crate::unpack_unit_e!(self.layout_size, "render before layout",);

        #[cfg(any(test, feature = "fuzztest"))]
        {
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

        // drawing the expanded nodes
        // TODO overflow
        for y in 0..std::cmp::min(self.selected_nodes.len() as u16, output.size().y) {
            // ticker
            {
                let begin_pos = XY::new(y * NESTED_MENU_FOLDER_WIDHT, y);
                output.print_at(begin_pos, expanded_nodes_style, NESTED_MENU_FOLDER_OPENED);

                // TODO do a limited write
                output.print_at(begin_pos + (1, 0), expanded_nodes_style, " ");
            }

            // the label
            {
                let begin_pos = XY::new((y + 1) * NESTED_MENU_FOLDER_WIDHT, y);
                let text = self.selected_nodes[y as usize].1.as_str();
                // TODO do a limited write
                output.print_at(begin_pos, expanded_nodes_style, text);

                // TODO overflow
                // drawing rest of whitespace
                for x in (begin_pos.x + text.width() as u16)..size.x {
                    output.print_at(XY::new(x, begin_pos.y), expanded_nodes_style, " ");
                }
            }
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

                for x in 0..x_offset {
                    output.print_at(XY::new(x, y_offset + idx), style, " ");
                }

                let begin_x = x_offset + 1 * NESTED_MENU_FOLDER_WIDHT;
                let text_pos = XY::new(begin_x, y_offset + idx);

                output.print_at(text_pos, style, item.label().as_ref());

                // coloring until end of size
                // TODO overflow
                for x in (begin_x + item.label().width() as u16)..size.x {
                    output.print_at(XY::new(x, text_pos.y), style, " ");
                }

                idx += 1;
            }
        }
    }
}
