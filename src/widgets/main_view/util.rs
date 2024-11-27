use log::warn;

use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::unpack_or_e;
use crate::widget::widget::Widget;

pub fn get_focus_path(root: &dyn Widget) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    fn internal(result: &mut Vec<String>, current_node: &dyn Widget) {
        if let Some(desc) = current_node.get_status_description() {
            result.push(desc.to_string());
        }

        if let Some(child) = current_node.get_focused() {
            internal(result, child);
        }
    }

    internal(&mut result, root);

    result
}

pub(crate) fn get_rect_for_context_menu(parent_size: XY, pos: XY) -> Option<Rect> {
    if !(pos < parent_size) {
        return None;
    }

    let mut rect = unpack_or_e!(Rect::hover_centered_percent(parent_size, 80), None, "failed to get centered rect");

    let mut xs = vec![(rect.min_x(), false), (rect.max_x(), false), (pos.x, true)];
    let mut ys = vec![(rect.min_y(), false), (rect.max_y(), false), (pos.y, true)];
    xs.sort_unstable();
    ys.sort_unstable();

    let mut idx_x = 0;
    let mut idx_y = 0;
    for idx in 0..3 {
        if xs[idx].1 {
            idx_x = idx;
        }
        if ys[idx].1 {
            idx_y = idx;
        }
    }

    let mut rect = Rect::from_zero(parent_size);

    if idx_x == 0 {
        rect.pos.x = xs[idx_x].0 + 1;
        if xs[2].0 <= rect.pos.x {
            warn!("cannot build rect for context_menu x 1");
            return None;
        }
        rect.size.x = xs[2].0 - rect.pos.x;
    } else if idx_x == 1 {
        let left = xs[1].0.abs_diff(xs[0].0) > xs[1].0.abs_diff(xs[2].0);

        if left {
            rect.pos.x = xs[0].0 + 1;
            if rect.pos.x >= xs[1].0 {
                warn!("cannot build rect for context_menu x 2");
                return None;
            }
            rect.size.x = xs[1].0 - rect.pos.x;
        } else {
            rect.pos.x = xs[1].0 + 1;
            if rect.pos.x >= xs[2].0 {
                warn!("cannot build rect for context_menu x 3");
                return None;
            }
            rect.size.x = xs[2].0 - rect.pos.x;
        }
    } else if idx_x == 2 {
        rect.pos.x = xs[0].0;
        rect.size.x = xs[2].0 - xs[0].0;
        if rect.size.x == 0 {
            warn!("cannot build rect for context_menu x 4");
            return None;
        }
    } else {
        debug_assert!(false, "impossible");
        return None;
    }

    if idx_y == 0 {
        rect.pos.y = ys[idx_y].0 + 1;
        if ys[2].0 <= rect.pos.y {
            warn!("cannot build rect for context_menu y 1");
            return None;
        }
        rect.size.y = ys[2].0 - rect.pos.y;
    } else if idx_y == 1 {
        let left = ys[1].0.abs_diff(ys[0].0) > ys[1].0.abs_diff(ys[2].0);

        if left {
            rect.pos.y = ys[0].0 + 1;
            if rect.pos.y >= ys[1].0 {
                warn!("cannot build rect for context_menu y 2");
                return None;
            }
            rect.size.y = ys[1].0 - rect.pos.y;
        } else {
            rect.pos.y = ys[1].0 + 1;
            if rect.pos.y >= ys[2].0 {
                warn!("cannot build rect for context_menu y 3");
                return None;
            }
            rect.size.y = ys[2].0 - rect.pos.y;
        }
    } else if idx_y == 2 {
        rect.pos.y = ys[0].0;
        rect.size.y = ys[2].0 - ys[0].0;
        if rect.size.y == 0 {
            warn!("cannot build rect for context_menu y 4");
            return None;
        }
    } else {
        debug_assert!(false, "impossible");
        return None;
    }

    Some(rect)
}
