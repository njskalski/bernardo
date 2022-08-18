use std::borrow::Borrow;

use crate::experiments::focus_group::FocusGroup;
use crate::experiments::from_geometry::get_focus_group;
use crate::layout::layout::Layout;
use crate::primitives::xy::XY;
use crate::Widget;

//TODO: more advanced option would store references to widgets instead of their WIDs.
// I'll consider that in a next step.

#[derive(Debug)]
pub struct GenericDisplayState {
    pub for_size: XY,
    pub focus_group: Box<dyn FocusGroup>,
}

impl GenericDisplayState {
    pub fn focus_group_mut(&mut self) -> &mut dyn FocusGroup {
        self.focus_group.as_mut()
    }

    pub fn focus_group(&self) -> &dyn FocusGroup {
        self.focus_group.borrow()
    }

    pub fn new<W: Widget>(root: &mut W, layout: &dyn Layout<W>, output_size: XY) -> Self {
        let focus_group = get_focus_group(root, layout, output_size);
        GenericDisplayState {
            for_size: output_size,
            focus_group: Box::new(focus_group),
        }
    }
}