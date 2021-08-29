/*
I was not able to design a "silver bullet" solution to views, messaging, layout and focus
without giving up on rust's safety warranties.

I decided that instead I will decouple the concerns, so same as layouting, focus is handled
in a separate component. When it's working the way I like it, I will then see if it can
get merged with some other component.
 */

use std::iter::Map;
use crate::primitives::rect::Rect;
use std::collections::HashMap;
use crate::io::keys::Key;
use crate::io::input_event::InputEvent;

#[derive(Hash, Eq, PartialEq)]
pub enum FocusUpdate {
    Left,
    Right,
    Up,
    Down,
    Next,
    Prev,
}

pub fn default_key_to_focus_update(key_input : InputEvent) -> Option<FocusUpdate> {
    match key_input {
        InputEvent::KeyInput(Key::ArrowLeft) => Some(FocusUpdate::Left),
        InputEvent::KeyInput(Key::ArrowRight) => Some(FocusUpdate::Right),
        InputEvent::KeyInput(Key::ArrowUp) => Some(FocusUpdate::Up),
        InputEvent::KeyInput(Key::ArrowDown) => Some(FocusUpdate::Down),
        InputEvent::KeyInput(Key::Tab) => Some(FocusUpdate::Next),
        // TODO handle shift tab somehow
        _ => None,
    }
}

pub trait FocusGroup {
    fn has_view(&self, widget_id : usize) -> bool;
    fn get_focused(&self) -> usize;

    /*
    returns whether focus got updated or not. It is designed to give a sound feedback, not for
    the purpose of escalation. There will be no "focus escalation".
     */
    fn update_focus(&mut self, focus_update : FocusUpdate) -> bool;

    //TODO proper error reporting
    fn override_edges(&mut self, widget_id : usize, edges : Vec<(FocusUpdate, usize)>) -> bool;
}

struct FocusGraphNode {
    widget_id : usize,
    neighbours : HashMap<FocusUpdate, usize>
}

impl FocusGraphNode {
    fn new(widget_id : usize) -> Self {
        FocusGraphNode {
            widget_id,
            neighbours: HashMap::new(),
        }
    }
}

pub struct FocusGroupImpl {
    nodes : HashMap<usize, FocusGraphNode>,
    selected : usize
}

impl FocusGroupImpl {
    pub fn new(widgets_and_positions : Vec<usize>) -> Self {
        let mut nodes = HashMap::<usize, FocusGraphNode>::new();
        
        for widget_id in widgets_and_positions.iter() {
            let node = FocusGraphNode::new(*widget_id);
            
            nodes.insert(*widget_id, node);
        };

        let selected  = widgets_and_positions.first().unwrap();

        FocusGroupImpl {
            nodes,
            selected : *selected,
        }
    }

    /*
    Takes a collection of pairs widget_id - rect and builds a directed graph, where nodes correspond
    to widgets, and edges A-e->B correspond to "which widget gets focus from A on input e".

    The graph is built using basic geometry.
     */
    //fn from_geometry(widgets_and_positions : Vec<(usize, Rect)>) -> Self {
}

impl FocusGroup for FocusGroupImpl {
    fn has_view(&self, widget_id: usize) -> bool {
        self.nodes.contains_key(&widget_id)
    }

    fn get_focused(&self) -> usize {
        self.selected
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

    fn override_edges(&mut self, widget_id: usize, edges: Vec<(FocusUpdate, usize)>) -> bool {
        match self.nodes.get_mut(&widget_id) {
            None => false,
            Some(node) => {
                node.neighbours.clear();
                for (e, v)  in edges {
                    node.neighbours.insert(e, v);
                }
                true
            }
        }
    }
}