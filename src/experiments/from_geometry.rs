/*
Takes a collection of pairs widget_id - rect and builds a directed graph, where nodes correspond
to widgets, and edges A-e->B correspond to "which widget gets focus from A on input e".

The graph is built using basic geometry.
 */
use crate::experiments::focus_group::FocusGroupImpl;
use crate::primitives::rect::Rect;
use crate::widget::widget::WID;


pub fn from_geometry(widgets_and_positions : Vec<(usize, Option<Rect>)>) -> FocusGroupImpl {
    let ids: Vec<WID> = widgets_and_positions.iter().map(|(wid, _)| *wid).collect();
    let fgi = FocusGroupImpl::new(ids);

    fgi
}