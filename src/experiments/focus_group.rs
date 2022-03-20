/*
I was not able to design a "silver bullet" solution to views, messaging, layout and focus
without giving up on rust's safety warranties.

I decided that instead I will decouple the concerns, so same as layouting, focus is handled
in a separate component. When it's working the way I like it, I will then see if it can
get merged with some other component.
 */

use std::collections::HashMap;
use std::fmt::Debug;

use log::error;

use crate::widget::widget::WID;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum FocusUpdate {
    Left,
    Right,
    Up,
    Down,
    Next,
    Prev,
}

pub trait FocusGroup: Debug {
    fn has_view(&self, widget_id: WID) -> bool;

    fn get_focused(&self) -> WID;
    fn set_focused(&mut self, wid: WID) -> bool;

    /*
    returns whether focus got updated or not. It is designed to give a sound feedback, not for
    the purpose of escalation. There will be no "focus escalation".
     */
    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool;

    fn can_update_focus(&self, focus_update: FocusUpdate) -> bool;

    //TODO proper error reporting
    fn override_edges(&mut self, widget_id: WID, edges: Vec<(FocusUpdate, WID)>) -> bool;
    fn add_edge(&mut self, src_widget: WID, edge: FocusUpdate, target_widget: WID) -> bool;

    // Removes item and drops all edges adjacent to it
    fn remove_item(&mut self, widget_id: WID) -> bool;
}

#[derive(Debug)]
struct FocusGraphNode {
    widget_id: WID,
    neighbours: HashMap<FocusUpdate, WID>,
}

impl FocusGraphNode {
    fn new(widget_id: WID) -> Self {
        FocusGraphNode {
            widget_id,
            neighbours: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct FocusGroupImpl {
    nodes: HashMap<WID, FocusGraphNode>,
    selected: usize,
}

impl FocusGroupImpl {
    pub fn new(widget_ids: Vec<WID>) -> Self {
        let mut nodes = HashMap::<WID, FocusGraphNode>::new();

        for widget_id in widget_ids.iter() {
            let node = FocusGraphNode::new(*widget_id);

            nodes.insert(*widget_id, node);
        }

        let selected = widget_ids.first().unwrap();

        FocusGroupImpl {
            nodes,
            selected: *selected,
        }
    }

    pub fn dummy() -> Self {
        FocusGroupImpl {
            nodes: HashMap::new(),
            selected: 0,
        }
    }
}

impl FocusGroup for FocusGroupImpl {
    fn has_view(&self, widget_id: WID) -> bool {
        self.nodes.contains_key(&widget_id)
    }

    fn get_focused(&self) -> WID {
        //debug!("get_focused : {}", self.selected);
        self.selected
    }

    fn set_focused(&mut self, wid: WID) -> bool {
        //debug!("set_focused : {}", wid);
        if self.has_view(wid) {
            self.selected = wid;
            true
        } else {
            false
        }
    }

    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
        let curr = self.nodes.get(&self.selected).unwrap();
        let next_op = curr.neighbours.get(&focus_update);

        match next_op {
            None => false,
            Some(next) => {
                self.selected = *next;
                debug_assert!(self.nodes.contains_key(next));
                true
            }
        }
    }

    fn can_update_focus(&self, focus_update: FocusUpdate) -> bool {
        let curr = self.nodes.get(&self.selected).unwrap();
        let next_op = curr.neighbours.get(&focus_update);

        next_op.is_some()
    }


    fn override_edges(&mut self, widget_id: WID, edges: Vec<(FocusUpdate, WID)>) -> bool {
        match self.nodes.get_mut(&widget_id) {
            None => false,
            Some(node) => {
                node.neighbours.clear();
                for (e, v) in edges {
                    node.neighbours.insert(e, v);
                }
                true
            }
        }
    }

    fn add_edge(&mut self, src_widget: WID, edge: FocusUpdate, target_widget: WID) -> bool {
        match self.nodes.get_mut(&src_widget) {
            None => false,
            Some(src_node) => {
                src_node.neighbours.insert(edge, target_widget);
                true
            }
        }
    }

    fn remove_item(&mut self, widget_id: WID) -> bool {
        if let Some(item) = self.nodes.remove(&widget_id) {
            for (_fu, id) in item.neighbours.iter() {
                self.nodes.get_mut(id).map(|node| {
                    let all_items: Vec<_> = node.neighbours.iter().map(|(a, b)| (*a, *b)).collect();
                    for (fu, neighbour) in all_items.iter() {
                        if *neighbour == widget_id {
                            node.neighbours.remove(fu);
                        }
                    }
                }).unwrap_or_else(|| {
                    error!("failed to read neighbour {} of {}", id, widget_id);
                });
            }

            true
        } else {
            false
        }
    }
}
