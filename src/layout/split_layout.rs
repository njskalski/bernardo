use crate::primitives::xy::XY;
use crate::layout::layout::{Layout};
use crate::primitives::rect::Rect;
use crate::experiments::focus_group::FocusUpdate;
use std::slice::Iter;
use crate::widget::widget::WID;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SplitDirection {
    Horizontal,
    Vertical
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SplitRule {
    Fixed(usize),
    Proportional(f32), // TODO change to float
}

pub struct SplitLayout {
    children : Vec<Box<dyn Layout>>,
    focused : usize,
    split_direction : SplitDirection,
    split_params : Vec<SplitRule>
}

impl SplitLayout {
    pub fn new(children : Vec<Box<dyn Layout>>,
               split_direction : SplitDirection,
               split_params : Vec<SplitRule>,
    ) -> Option<Self> {
        if children.is_empty() {
            return None
        }

        if children.len() != split_params.len() {
            return None
        }

        for p in split_params.iter() {
            match p {
                SplitRule::Fixed(_) => {}
                SplitRule::Proportional(p) => {
                    if *p <= 0.0 {
                        return None
                    }
                }
            }
        }

        Some(SplitLayout {
            children,
            focused: 0,
            split_direction,
            split_params,
        })
    }

    fn id_to_idx(&self, widget_id : WID) -> Option<usize> {
        for i in 0..self.children.len() {
            if self.children[i].has_id(widget_id) {
                return Some(i)
            }
        }
        None
    }

    fn get_rects(&self, size: XY) -> Option<Vec<Rect>> {
        let free_axis = if self.split_direction == SplitDirection::Vertical {
            size.y as usize
        } else {
            size.x as usize
        };

        let fixed_amount = self.split_params.iter().fold(0,
        |acc, item| acc + match item {
            SplitRule::Fixed(i) => *i,
            SplitRule::Proportional(_) => 0,
        });

        if fixed_amount > free_axis {
            return None
        }

        let leftover = free_axis - fixed_amount;
        let mut amounts : Vec<usize> = vec![0; self.split_params.len()];

        let mut sum_props = 0.0f32;

        for (idx, rule) in self.split_params.iter().enumerate() {
            match rule {
                SplitRule::Fixed(f) => {
                    amounts[idx] = *f;
                }
                SplitRule::Proportional(prop) => {
                    sum_props += prop;
                }
            }
        }

        let unit = leftover as f32 / sum_props;

        for (idx, rule) in self.split_params.iter().enumerate() {
            if let SplitRule::Proportional(p) = rule {
                amounts[idx] = (unit * p) as usize;
            }
        }

        let mut res : Vec<Rect> = Vec::new();
        res.reserve(amounts.len());

        let mut upper_left = XY::new(0, 0);

        for s in amounts.iter() {
            let new_size: XY = if self.split_direction == SplitDirection::Vertical {
                (size.x, *s as u16).into()
            } else {
                (*s as u16, size.y).into()
            };

            res.push(Rect::new(upper_left, new_size));

            upper_left = upper_left + if self.split_direction == SplitDirection::Vertical {
                XY::new(0, *s as u16)
            } else {
                XY::new(*s as u16, 0).into()
            };
        };
        Some(res)
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
    fn get_rect(&self, output_size: XY, widget_id: WID) -> Option<Rect> {
        let idx_op = self.id_to_idx(widget_id);
        if idx_op.is_none() {
            return None
        }
        let idx = idx_op.unwrap();

        let rects_op = self.get_rects(output_size);
        if rects_op.is_none() {
            return None
        }
        let rects = rects_op.unwrap();

        let child_rec = self.children[idx].get_rect(rects[idx].size, widget_id);
        child_rec.map(|r| r.shift(rects[idx].pos))
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn has_id(&self, widget_id: WID) -> bool {
        for layout in self.children.iter() {
            if layout.has_id(widget_id) {
                return true
            }
        }

        false
    }

    fn get_ids(&self) -> Vec<WID> {
        self.children.iter().flat_map(|c| c.get_ids()).collect()
    }
}

