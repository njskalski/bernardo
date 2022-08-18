/*
Takes a collection of pairs widget_id - rect and builds a directed graph, where nodes correspond
to widgets, and edges A-e->b correspond to "which widget gets focus from A on input e".

The graph is built using basic geometry.
 */
use std::collections::HashMap;

use crate::experiments::focus_group::{FocusGroup, FocusGroupImpl, FocusUpdate};
use crate::io::buffer::Buffer;
use crate::layout::layout::Layout;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::Widget;
use crate::widget::widget::WID;

// use log::debug;

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

        for mut w in &mut walkers {
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
                    None => (0 as usize, u16::MAX),
                    Some(p) => p.clone(),
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

pub fn get_focus_group<W: Widget>(root: &mut W, layout: &dyn Layout<W>, output_size: XY) -> FocusGroupImpl {
    let wwrs = layout.layout(root, output_size);

    let widgets_and_positions: Vec<(WID, Rect)> = wwrs.into_iter().map(|w| {
        let (swp, rect) = w.unpack();
        let wid = swp.get(root).id();
        (wid, rect)
    }).collect();

    from_geometry(&widgets_and_positions, Some(output_size))
}

fn get_full_size(widgets_and_positions: &Vec<(WID, Rect)>) -> XY {
    let mut full_size = XY::new(0, 0);

    for (_, rect) in widgets_and_positions {
        if full_size.x < rect.max_x() {
            full_size.x = rect.max_x();
        }
        if full_size.y < rect.max_y() {
            full_size.y = rect.max_y();
        }
    }

    full_size
}

pub fn from_geometry(
    widgets_and_positions: &Vec<(WID, Rect)>,
    output_size_op: Option<XY>,
) -> FocusGroupImpl {
    let ids: Vec<WID> = widgets_and_positions.iter().map(|(wid, _)| *wid).collect();
    let mut fgi = FocusGroupImpl::new(ids);

    let output_size = output_size_op.unwrap_or(get_full_size(&widgets_and_positions));

    let mut buffer: Buffer<WID> = Buffer::new(output_size);

    for (wid, rect) in widgets_and_positions {
        fill(&mut buffer, *wid, &rect);
    }

    for (wid, rect) in widgets_and_positions {
        let mut edges: Vec<(FocusUpdate, WID)> = Vec::new();

        match walk_to_first_hit(&buffer, *wid, rect, (-1, 0)) {
            Some(left) => edges.push((FocusUpdate::Left, left)),
            None => {}
        };
        match walk_to_first_hit(&buffer, *wid, rect, (1, 0)) {
            Some(right) => edges.push((FocusUpdate::Right, right)),
            None => {}
        };
        match walk_to_first_hit(&buffer, *wid, rect, (0, 1)) {
            Some(down) => edges.push((FocusUpdate::Down, down)),
            None => {}
        };
        match walk_to_first_hit(&buffer, *wid, rect, (0, -1)) {
            Some(up) => edges.push((FocusUpdate::Up, up)),
            None => {}
        };

        fgi.override_edges(*wid, edges);
    }

    fgi
}

#[cfg(test)]
mod tests {
    use crate::experiments::focus_group::{FocusGroup, FocusUpdate};
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
        let widgets_and_positions: Vec<(WID, Rect)> = vec![
            (1, Rect::new((2, 0).into(), (1, 1).into())),
            (2, Rect::new((0, 2).into(), (1, 1).into())),
            (3, Rect::new((2, 2).into(), (1, 1).into())),
            (4, Rect::new((4, 2).into(), (1, 1).into())),
            (5, Rect::new((2, 4).into(), (3, 1).into())),
        ];

        let mut focus_group = from_geometry(&widgets_and_positions, None);

        assert_eq!(focus_group.get_focused(), 1);
        assert_eq!(focus_group.update_focus(FocusUpdate::Left), false);
        assert_eq!(focus_group.update_focus(FocusUpdate::Right), false);
        assert_eq!(focus_group.update_focus(FocusUpdate::Up), false);

        assert_eq!(focus_group.update_focus(FocusUpdate::Down), true);
        assert_eq!(focus_group.get_focused(), 3);
        assert_eq!(focus_group.update_focus(FocusUpdate::Up), true);
        assert_eq!(focus_group.get_focused(), 1);
        assert_eq!(focus_group.update_focus(FocusUpdate::Down), true);
        assert_eq!(focus_group.get_focused(), 3);
        assert_eq!(focus_group.update_focus(FocusUpdate::Left), true);
        assert_eq!(focus_group.get_focused(), 2);
        assert_eq!(focus_group.update_focus(FocusUpdate::Up), false);
        assert_eq!(focus_group.update_focus(FocusUpdate::Down), false);
        assert_eq!(focus_group.update_focus(FocusUpdate::Right), true);
        assert_eq!(focus_group.get_focused(), 3);
        assert_eq!(focus_group.update_focus(FocusUpdate::Right), true);
        assert_eq!(focus_group.get_focused(), 4);
        assert_eq!(focus_group.update_focus(FocusUpdate::Up), false);
        assert_eq!(focus_group.update_focus(FocusUpdate::Down), true);
        assert_eq!(focus_group.get_focused(), 5);
        assert_eq!(focus_group.update_focus(FocusUpdate::Left), false);
        assert_eq!(focus_group.update_focus(FocusUpdate::Right), false);
        assert_eq!(focus_group.update_focus(FocusUpdate::Down), false);

        assert_eq!(focus_group.update_focus(FocusUpdate::Up), true);
        assert_eq!(focus_group.get_focused(), 3);
    }
}
