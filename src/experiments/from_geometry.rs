/*
Takes a collection of pairs widget_id - rect and builds a directed graph, where nodes correspond
to widgets, and edges A-e->b correspond to "which widget gets focus from A on input e".

The graph is built using basic geometry.
 */
use std::cmp::max;
use std::collections::HashMap;

use crate::experiments::focus_group::{FocusGraph, FocusGraphNode, FocusUpdate};
use crate::io::buffer::Buffer;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::widget::WID;

fn fill(b: &mut Buffer<WID>, wid: WID, rect: &Rect) {
    for x in rect.min_x()..u16::min(rect.max_x(), b.size().x) {
        for y in rect.min_y()..u16::min(rect.max_y(), b.size().y) {
            let xy = XY::new(x, y);
            // debug!("xy {}, rect {}", xy, rect);
            b[xy] = wid;
        }
    }
}

fn walk_to_first_hit(buffer: &Buffer<WID>, wid: WID, rect: &Rect, step: (i16, i16)) -> Option<WID> {
    // right wall
    let mut walkers: Vec<XY> = Vec::new();

    if step == (-1, 0) {
        for y in rect.min_y()..rect.max_y() {
            walkers.push(XY::new(rect.min_x(), y))
        }
    }
    if step == (1, 0) && rect.max_x() > 0 {
        for y in rect.min_y()..rect.max_y() {
            walkers.push(XY::new(rect.max_x() - 1, y))
        }
    }
    if step == (0, -1) {
        for x in rect.min_x()..rect.max_x() {
            walkers.push(XY::new(x, rect.min_y()))
        }
    }
    if step == (0, 1) && rect.max_y() > 0 {
        for x in rect.min_x()..rect.max_x() {
            walkers.push(XY::new(x, rect.max_y() - 1))
        }
    }

    'outer: loop {
        // WID -> overlap, lowest_index is overlap > 0 or maxint
        let mut hits: HashMap<WID, (usize, u16)> = HashMap::new();

        for w in &mut walkers {
            if step.0 == -1 && w.x == 0 {
                break 'outer;
            }
            if step.1 == -1 && w.y == 0 {
                break 'outer;
            }

            w.x = (w.x as i16 + step.0) as u16;
            w.y = (w.y as i16 + step.1) as u16;

            if !buffer.within(*w) {
                break 'outer;
            }

            let dst = buffer[*w];
            if dst != 0 && dst != wid {
                let mut prev = match hits.get(&dst) {
                    None => (0, u16::MAX),
                    Some(p) => *p,
                };

                prev.0 += 1;
                if step.1 != 0 && prev.1 > w.x {
                    prev.1 = w.x;
                }
                if step.0 != 0 && prev.1 > w.y {
                    prev.1 = w.y;
                }

                hits.insert(dst, prev);
            }
        }

        let mut best_owid: usize = 0;
        let mut best_hit: (usize, u16) = (0, u16::MAX);

        for (owid, hit) in &hits {
            if *owid == wid {
                continue;
            }

            if hit.0 > best_hit.0 || (hit.0 == best_hit.0 && hit.1 < best_hit.1) {
                best_hit = *hit;
                best_owid = *owid;
            }
        }

        if best_owid != 0 {
            return Some(best_owid);
        }
    }

    None
}

fn get_full_size(widgets_and_positions: &Vec<(WID, Rect)>) -> XY {
    let mut full_size = XY::new(0, 0);

    for (_, rect) in widgets_and_positions {
        full_size.x = max(full_size.x, rect.max_x());
        full_size.y = max(full_size.y, rect.max_y());
    }

    full_size
}

// TODO add test WID in w_a_p..
pub fn from_geometry<AdditionalData: Clone>(
    widgets_and_positions: &Vec<(WID, AdditionalData, Rect)>,
    selected: WID,
    output_size: XY,
) -> FocusGraph<AdditionalData> {
    // let ids: Vec<WID> = widgets_and_positions.iter().map(|(wid, _, _)| *wid).collect();
    let mut nodes: HashMap<WID, FocusGraphNode<AdditionalData>> = HashMap::default();

    for (id, add, _) in widgets_and_positions.iter() {
        nodes.insert(*id, FocusGraphNode::new(*id, add.clone()));
    }

    let mut fgi = FocusGraph::<AdditionalData>::new(nodes, selected);
    let mut buffer: Buffer<WID> = Buffer::new(output_size);

    for (wid, _, rect) in widgets_and_positions.iter() {
        fill(&mut buffer, *wid, rect);
    }

    for (source, _, rect) in widgets_and_positions {
        let mut edges: Vec<(FocusUpdate, WID)> = Vec::new();

        if let Some(left) = walk_to_first_hit(&buffer, *source, rect, (-1, 0)) {
            edges.push((FocusUpdate::Left, left))
        }
        if let Some(right) = walk_to_first_hit(&buffer, *source, rect, (1, 0)) {
            edges.push((FocusUpdate::Right, right))
        }
        if let Some(down) = walk_to_first_hit(&buffer, *source, rect, (0, 1)) {
            edges.push((FocusUpdate::Down, down))
        }
        if let Some(up) = walk_to_first_hit(&buffer, *source, rect, (0, -1)) {
            edges.push((FocusUpdate::Up, up))
        }

        for (focus_update, target) in edges {
            fgi.add_edge(*source, focus_update, target);
        }
    }

    fgi
}

#[cfg(test)]
mod tests {
    use crate::experiments::focus_group::FocusUpdate;
    use crate::experiments::from_geometry::{from_geometry, get_full_size};
    use crate::primitives::rect::Rect;
    use crate::primitives::xy::XY;
    use crate::widget::widget::WID;

    #[test]
    fn full_size_test() {
        /*
           ##1##
           #####
           2#3#4
           #####
           ##555
        */

        //widgets_and_positions : Vec<(WID, Option<Rect>)>, output_size : XY
        let widgets_and_positions: Vec<(WID, Rect)> = vec![
            (1, Rect::new((2, 0).into(), (1, 1).into())),
            (2, Rect::new((0, 2).into(), (1, 1).into())),
            (3, Rect::new((2, 2).into(), (1, 1).into())),
            (4, Rect::new((4, 2).into(), (1, 1).into())),
            (5, Rect::new((2, 4).into(), (3, 1).into())),
        ];

        let full_size = get_full_size(&widgets_and_positions);

        assert_eq!(full_size, XY::new(5, 5));
    }

    #[test]
    fn from_geometry_test() {
        /*
           ##1##
           #####
           2#3#4
           #####
           ##555
        */

        //widgets_and_positions : Vec<(WID, Option<Rect>)>, output_size : XY
        let widgets_and_positions: Vec<(WID, (), Rect)> = vec![
            (1, (), Rect::new((2, 0).into(), (1, 1).into())),
            (2, (), Rect::new((0, 2).into(), (1, 1).into())),
            (3, (), Rect::new((2, 2).into(), (1, 1).into())),
            (4, (), Rect::new((4, 2).into(), (1, 1).into())),
            (5, (), Rect::new((2, 4).into(), (3, 1).into())),
        ];

        let mut focus_group = from_geometry::<()>(&widgets_and_positions, 1, XY::new(5, 5));

        assert_eq!(focus_group.get_focused_id(), 1);
        assert!(!focus_group.update_focus(FocusUpdate::Left));
        assert!(!focus_group.update_focus(FocusUpdate::Right));
        assert!(!focus_group.update_focus(FocusUpdate::Up));

        assert!(focus_group.update_focus(FocusUpdate::Down));
        assert_eq!(focus_group.get_focused_id(), 3);
        assert!(focus_group.update_focus(FocusUpdate::Up));
        assert_eq!(focus_group.get_focused_id(), 1);
        assert!(focus_group.update_focus(FocusUpdate::Down));
        assert_eq!(focus_group.get_focused_id(), 3);
        assert!(focus_group.update_focus(FocusUpdate::Left));
        assert_eq!(focus_group.get_focused_id(), 2);
        assert!(!focus_group.update_focus(FocusUpdate::Up));
        assert!(!focus_group.update_focus(FocusUpdate::Down));
        assert!(focus_group.update_focus(FocusUpdate::Right));
        assert_eq!(focus_group.get_focused_id(), 3);
        assert!(focus_group.update_focus(FocusUpdate::Right));
        assert_eq!(focus_group.get_focused_id(), 4);
        assert!(!focus_group.update_focus(FocusUpdate::Up));
        assert!(focus_group.update_focus(FocusUpdate::Down));
        assert_eq!(focus_group.get_focused_id(), 5);
        assert!(!focus_group.update_focus(FocusUpdate::Left));
        assert!(!focus_group.update_focus(FocusUpdate::Right));
        assert!(!focus_group.update_focus(FocusUpdate::Down));

        assert!(focus_group.update_focus(FocusUpdate::Up));
        assert_eq!(focus_group.get_focused_id(), 3);
    }
}
