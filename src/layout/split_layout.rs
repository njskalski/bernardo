use crate::primitives::xy::XY;
use crate::layout::layout::{Layout, FocusUpdate};
use crate::primitives::rect::Rect;

pub struct SplitLayout {
    children : Vec<usize>,
    focused : usize,
    split_directions : XY,
}

impl SplitLayout {

    pub fn new(children : Vec<usize>, split_directions : XY) -> Option<Self> {
        if children.is_empty() {
            return None
        }

        if split_directions.x * split_directions.y != children.len() as u16 {
            return None
        }

        Some(SplitLayout {
            children,
            focused: 0,
            split_directions
        })
    }

    fn id_to_idx(&self, id : usize) -> Option<usize> {
        for i in 0..self.children.len() {
            if self.children[i] == id {
                return Some(i)
            }
        }
        None
    }

    // we calculate 1,2,3,4
    //              5,6,7,8...
    fn idx_to_coords(&self, idx : usize) -> XY {
        assert!(idx < self.children.len());

        let y = self.children.len() / self.split_directions.x;
        let x = self.children.len() % self.split_directions.x;

        (x, y).into()
    }

}

impl Layout for SplitLayout {
    fn get_focused(&self) -> usize {
        self.focused
    }

    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
        //TODO
        self.focused = self.focused % self.children.len();
        true
    }

    /*
    output_size: tells how big output we have. This is used by Layouts that adjust to output size.
     */
    fn get_rect(&self, output_size: XY, widget_id: usize) -> Option<Rect> {
        let idx_op = self.id_to_idx(widget_id);

        if idx_op.is_none() {
            return None
        }

        let idx = idx_op.unwrap();

        let pos = self.idx_to_coords(idx);

        let x_unit = output_size.x / self.split_directions.x;
        let y_unit = output_size.y / self.split_directions.y;

        let upper_left = XY::new(x_unit * pos.x, y_unit * pos.y);
        Some(Rect::new(upper_left, (x_unit, y_unit).into()))
    }
}

