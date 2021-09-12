use crate::layout::layout::Layout;
use crate::primitives::rect::Rect;
use crate::experiments::focus_group::{FocusUpdate, FocusGroup};
use crate::primitives::xy::XY;
use crate::widget::widget::WID;
use std::collections::HashMap;
use crate::experiments::from_geometry::from_geometry;

pub struct FixedItem {
    pub layout : Box<dyn Layout>,
    pub rect : Rect
}

impl FixedItem {
    pub fn new(layout : Box<dyn Layout>, rect : Rect) -> Self {
        FixedItem {
            layout,
            rect,
        }
    }
}

pub struct FixedLayout {
    size : XY,
    items : Vec<FixedItem>,
    focus : usize,
    focus_group : Box<dyn FocusGroup>,
}

impl FixedLayout {
    pub fn new(size : XY, items : Vec<FixedItem>) -> Self {

        let all_items : Vec<(WID, Option<Rect>)> = items.iter()
            .flat_map(|f| f.layout.get_all(size)).collect();

        let fg = from_geometry(&all_items, Some(size));

        FixedLayout {
            size,
            items,
            focus : 0,
            focus_group : Box::new(fg),
        }
    }
}

impl Layout for FixedLayout {
    fn get_focused(&self) -> usize {
        self.items[self.focus].layout.get_focused()
    }

    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
        self.focus_group.update_focus(focus_update)
    }

    fn get_rect(&self, output_size: XY, widget_id: WID) -> Option<Rect> {
        match self.items.iter().find(|item| item.layout.has_id(widget_id)) {
            None => None,
            Some(right_item) => match right_item.layout.get_rect(right_item.rect.size, widget_id) {
                None => None,
                Some(mut rect) => {
                    rect.pos = rect.pos + right_item.rect.pos;
                    if rect.max_xy() <= output_size {
                        Some(rect)
                    } else {
                        None
                    }
                }
            }
        }
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn has_id(&self, widget_id: WID) -> bool {
        for fi in self.items.iter() {
            if fi.layout.has_id(widget_id) {
                return true;
            }
        }
        false
    }

    fn get_ids(&self) -> Vec<WID> {
        self.items.iter().flat_map(|f| f.layout.get_ids()).collect()
    }
}