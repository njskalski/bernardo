use crate::primitives::xy::XY;
use crate::layout::layout::{Layout, WidgetGetterMut};
use crate::primitives::rect::Rect;
use crate::experiments::focus_group::FocusUpdate;
use std::slice::Iter;
use crate::widget::widget::{WID, Widget};
use crate::io::output::Output;
use std::net::Shutdown::Read;
use log::debug;
use log::warn;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SplitDirection {
    Horizontal,
    Vertical
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SplitRule {
    Fixed(usize),
    Proportional(f32),
}

pub struct SplitLayout<W : Widget> {
    children : Vec<Box<dyn Layout<W>>>,
    focused : usize,
    split_direction : SplitDirection,
    split_params : Vec<SplitRule>
}

impl <W: Widget> SplitLayout<W> {
    pub fn new(split_direction : SplitDirection) -> Self {
        SplitLayout {
            children: vec![],
            focused: 0,
            split_direction,
            split_params: vec![]
        }
    }

    // pub fn with(self,
    // ) -> Self {
    //
    // }

    pub fn old_new(children : Vec<Box<dyn Layout<W>>>,
               split_direction : SplitDirection,
               split_params : Vec<SplitRule>) -> Option<Self> {
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

    // fn id_to_idx(&self, widget_id : WID) -> Option<usize> {
    //     for i in 0..self.children.len() {
    //         if self.children[i].has_id(widget_id) {
    //             return Some(i)
    //         }
    //     }
    //     None
    // }

    fn get_rects(&self, size: XY) -> Option<Vec<Rect>> {
        debug_assert!(self.children.len() == self.split_params.len());

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

impl <W: Widget> Layout<W> for SplitLayout<W> {
    fn get_focused<'a>(&self, parent: &'a W) -> &'a dyn Widget {
        self.children[self.focused].get_focused(parent)
    }

    fn get_focused_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget {
        self.children[self.focused].get_focused_mut(parent)
    }

    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
        //TODO
        self.focused = self.focused % self.children.len();
        true
    }

    /*
    output_size: tells how big output we have. This is used by Layouts that adjust to output size.
     */
    // fn get_rect(&self, output_size: XY, widget_id: WID) -> Option<Rect> {
    //     let idx_op = self.id_to_idx(widget_id);
    //     if idx_op.is_none() {
    //         return None
    //     }
    //     let idx = idx_op.unwrap();
    //
    //     let rects_op = self.get_rects(output_size);
    //     if rects_op.is_none() {
    //         return None
    //     }
    //     let rects = rects_op.unwrap();
    //
    //     let child_rec = self.children[idx].get_rect(rects[idx].size, widget_id);
    //     child_rec.map(|r| r.shift(rects[idx].pos))
    // }

    // fn is_leaf(&self) -> bool {
    //     false
    // }

    // fn has_id(&self, widget_id: WID) -> bool {
    //     for layout in self.children.iter() {
    //         if layout.has_id(widget_id) {
    //             return true
    //         }
    //     }
    //
    //     false
    // }

    // fn get_ids(&self) -> Vec<WID> {
    //     self.children.iter().flat_map(|c| c.get_ids()).collect()
    // }

    fn render(&self, owner: &W, focused_id: Option<WID>, output: &mut Output) {
        let rects_op = self.get_rects(output.size());

        if rects_op.is_none() {
            warn!("not enough space to draw split_layout: {:?}", output.size());
            return;
        }

        let rects = rects_op.unwrap();

        let visible_rect = output.get_visible_rect();

        for idx in 0..rects.len() {
            let child = &self.children[idx];
            let rect = &rects[idx];

            if visible_rect.intersect(rect).is_some() {
                child.render(owner, focused_id, output);
            } else {
                debug!("skipping drawing split_layout item {:} {:?} beause it's outside view {:?}",
                    idx, rect, visible_rect);
            }
        }
    }
}

