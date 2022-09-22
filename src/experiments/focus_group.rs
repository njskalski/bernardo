use std::collections::HashMap;
use std::fmt::Debug;

use log::error;
use serde::{Deserialize, Serialize};

use crate::widget::widget::WID;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FocusUpdate {
    Left,
    Right,
    Up,
    Down,
    Next,
    Prev,
}


#[derive(Clone, Debug)]
pub struct FocusGraphNode<AdditionalData: Clone> {
    widget_id: WID,
    additional_data: AdditionalData,
    neighbours: HashMap<FocusUpdate, WID>,
}

impl<AdditionalData: Clone> FocusGraphNode<AdditionalData> {
    pub fn new(widget_id: WID, add: AdditionalData) -> Self {
        FocusGraphNode {
            widget_id,
            additional_data: add,
            neighbours: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FocusGraph<AdditionalData: Clone> {
    nodes: HashMap<WID, FocusGraphNode<AdditionalData>>,
    selected: WID,
}

impl<AdditionalData: Clone> FocusGraph<AdditionalData> {
    pub fn new(nodes: HashMap<WID, FocusGraphNode<AdditionalData>>, selected: WID) -> Self {
        // TODO
        // debug_assert!(nodes.keys().fold(false, |a, b| a || *b == selected), "selected node present TODO not sure if that is what that asertion does");

        Self {
            nodes,
            selected,
        }
    }

    pub fn add_edge(&mut self, source: WID, edge: FocusUpdate, target: WID) -> bool {
        if !self.nodes.contains_key(&target) {
            error!("target node {} not found!", source);
            return false;
        }

        if let Some(node) = self.nodes.get_mut(&source) {
            node.neighbours.insert(edge, target);
            true
        } else {
            error!("source node {} not found!", source);
            false
        }
    }

    pub fn can_update_focus(&self, edge: FocusUpdate) -> bool {
        if let Some(node) = self.nodes.get(&self.selected) {
            node.neighbours.contains_key(&edge)
        } else {
            error!("self.selected node {} not found!", self.selected);
            false
        }
    }

    pub fn get_focused_id(&self) -> WID {
        self.selected
    }

    pub fn get_focused(&self) -> AdditionalData {
        self.nodes.get(&self.selected).map(|n| n.additional_data.clone()).unwrap()
    }

    pub fn update_focus(&mut self, edge: FocusUpdate) -> bool {
        if let Some(node) = self.nodes.get(&self.selected) {
            if let Some(target_id) = node.neighbours.get(&edge) {
                self.selected = *target_id;
                true
            } else {
                false
            }
        } else {
            error!("self.selected node {} not found!", self.selected);
            false
        }
    }
}