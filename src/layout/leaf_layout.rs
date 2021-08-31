use crate::layout::layout::{Layout};
use crate::primitives::xy::XY;
use crate::primitives::rect::Rect;
use crate::widget::widget::BaseWidget;
use crate::experiments::focus_group::FocusUpdate;

pub struct LeafLayout {
    widget_id : usize,
}

impl LeafLayout {
    pub fn new(widget_id : usize) -> Self {
        LeafLayout{
            widget_id
        }
    }

    pub fn from_widget(base_widget : &dyn BaseWidget) -> Self {
        LeafLayout {
            widget_id : base_widget.id()
        }
    }
}

impl Layout for LeafLayout {
    fn get_focused(&self) -> usize {
        self.widget_id
    }

    fn update_focus(&mut self, focus_update: &FocusUpdate) -> bool {
        false
    }

    fn get_rect(&self, output_size: XY, widget_id: usize) -> Option<Rect> {
        if self.widget_id == widget_id {
            Some(Rect::new(XY::new(0,0), output_size))
        } else {
            None
        }
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn has_id(&self, widget_id: usize) -> bool {
        self.widget_id == widget_id
    }
}