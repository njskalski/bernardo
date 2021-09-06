/*
Takes a collection of pairs widget_id - rect and builds a directed graph, where nodes correspond
to widgets, and edges A-e->B correspond to "which widget gets focus from A on input e".

The graph is built using basic geometry.
 */
use crate::experiments::focus_group::{FocusGroupImpl, FocusUpdate, FocusGroup};
use crate::primitives::rect::Rect;
use crate::widget::widget::WID;
use crate::primitives::xy::XY;
use crate::io::buffer::Buffer;
use std::collections::HashMap;

const directions : Vec<XY> = vec![
    XY::new(-1, 0), // left
    XY::new(1, 0), // right
    XY::new(0, -1), // up
    XY::new(0, 1), // down
];

fn fill(b : &mut Buffer<WID>, wid : WID, rect : &Rect) {
    for x in rect.min_x()..rect.max_x() {
        for y in rect.min_y()..rect.max_x() {
            let xy = XY::new(x,y);
            b[xy] = wid;
        }
    }
}

fn walk_to_first_hit(buffer : &Buffer<WID>, step : XY) -> Option<WID> {
    // right wall
    let mut walkers : Vec<XY> = Vec::new();

    if step == XY::new(-1, 0) {
        for y in rect.min_y()..rect.max_y() {
            walkers.push(XY::new(rect.min_x(), y))
        }
    }
    if step == XY::new(1, 0) {
        for y in rect.min_y()..rect.max_y() {
            walkers.push(XY::new(rect.max_x()-1, y))
        }
    }
    if step == XY::new(0, -1) {
        for x in rect.min_x()..rect.max_x() {
            walkers.push(XY::new(x, rect.min_y()))
        }
    }
    if step == XY::new(0, 1) {
        for x in rect.min_x()..rect.max_x() {
            walkers.push(XY::new(x, rect.min_y()-1))
        }
    }

    'outer : loop {
        let mut hits : HashMap<WID, usize> = HashMap::new();

        for mut w in walkers {
            if step.x == -1 && w.x == 0 {
                break 'outer
            }
            if step.y == -1 && w.y == 0 {
                break 'outer
            }

            w += step;

            if !buffer.within(w) {
                break 'outer
            }

            if buffer[w] != 0 && buffer[w] != wid {
                hits[buffer[w]] += 1;
            }
        }

        let mut best : usize = 0;
        let mut best_hits : usize = 0;

        for (owid, num_hits) in hits {
            if num_hits > best_hits && owid != wid {
                best_hits = num_hits;
                best = owid;
            }
        }

        if best != 0 {
            return Some(owid)
        }
    }

    None
}

pub fn from_geometry(widgets_and_positions : Vec<(WID, Option<Rect>)>, output_size : XY) -> FocusGroupImpl {
    let ids: Vec<WID> = widgets_and_positions.iter().map(|(wid, _)| *wid).collect();
    let mut fgi = FocusGroupImpl::new(ids);

    let mut buffer : Buffer<WID> = Buffer::new(output_size);

    for (wid, rect_op) in widgets_and_positions {
        match rect_op {
            Some(rect) => fill(&mut buffer, wid, &rect),
            None => {}
        }
    }

    for (wid, rect_op) in widgets_and_positions {
        let mut edges: Vec<(FocusUpdate, WID)> = Vec::new();
        if rect_op.is_some() {
            match walk_to_first_hit(&buffer, XY::new(-1, 0)) {
                Some(left) => edges.push((FocusUpdate::Left, left)),
                None => {}
            };
            match walk_to_first_hit(&buffer, XY::new(1, 0)) {
                Some(right) => edges.push((FocusUpdate::Right, right)),
                None => {}
            };
            match walk_to_first_hit(&buffer, XY::new(0, 1)) {
                Some(up) => edges.push((FocusUpdate::Up, up)),
                None => {}
            };
            match walk_to_first_hit(&buffer, XY::new(0, -1)) {
                Some(down) => edges.push((FocusUpdate::Down, down)),
                None => {}
            };
        }

        fgi.override_edges(wid, edges);
    }

    fgi
}

#[cfg(test)]
mod tests {
    use crate::experiments::from_geometry::from_geometry;

    #[test]
    fn sometest() {
        // let ss = simple_styled_string("hello world");
        //
        // assert_eq!(ss.is_flat(), true);
        // assert_eq!(ss.len(), 11);
        // assert_eq!(ss.size(), XY::new(11,1));




    }

}