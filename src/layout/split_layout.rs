use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::layout::layout::{Layout, WidgetGetterMut};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{Widget, WID};
use log::debug;
use log::warn;
use std::net::Shutdown::Read;
use std::slice::Iter;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SplitRule {
    Fixed(usize),
    Proportional(f32),
}

struct SplitLayoutChild<W: Widget> {
    layout: Box<dyn Layout<W>>,
    split_rule: SplitRule,
}

pub struct SplitLayout<W: Widget> {
    children: Vec<SplitLayoutChild<W>>,
    focused: usize,
    split_direction: SplitDirection,
}

impl<W: Widget> SplitLayout<W> {
    pub fn new(split_direction: SplitDirection) -> Self {
        SplitLayout {
            children: vec![],
            focused: 0,
            split_direction,
        }
    }

    // pub fn with(self,
    // ) -> Self {
    //
    // }

    // pub fn old_new(children : Vec<Box<dyn Layout<W>>>,
    //            split_direction : SplitDirection,
    //            split_params : Vec<SplitRule>) -> Option<Self> {
    //     if children.is_empty() {
    //         return None
    //     }
    //
    //     if children.len() != split_params.len() {
    //         return None
    //     }
    //
    //     for p in split_params.iter() {
    //         match p {
    //             SplitRule::Fixed(_) => {}
    //             SplitRule::Proportional(p) => {
    //                 if *p <= 0.0 {
    //                     return None
    //                 }
    //             }
    //         }
    //     }
    //
    //     Some(SplitLayout {
    //         children,
    //         focused: 0,
    //         split_direction,
    //     })
    // }

    fn get_rects(&self, size: XY) -> Option<Vec<Rect>> {
        let free_axis = if self.split_direction == SplitDirection::Vertical {
            size.y as usize
        } else {
            size.x as usize
        };

        let fixed_amount = self.children.iter().fold(0, |acc, item| {
            acc + match item.split_rule {
                SplitRule::Fixed(i) => i,
                SplitRule::Proportional(_) => 0,
            }
        });

        if fixed_amount > free_axis {
            return None;
        }

        let leftover = free_axis - fixed_amount;
        let mut amounts: Vec<usize> = vec![0; self.children.len()];

        let mut sum_props = 0.0f32;

        for (idx, child) in self.children.iter().enumerate() {
            match child.split_rule {
                SplitRule::Fixed(f) => {
                    amounts[idx] = f;
                }
                SplitRule::Proportional(prop) => {
                    sum_props += prop;
                }
            }
        }

        let unit = leftover as f32 / sum_props;

        for (idx, child) in self.children.iter().enumerate() {
            if let SplitRule::Proportional(p) = child.split_rule {
                amounts[idx] = (unit * p) as usize;
            }
        }

        let mut res: Vec<Rect> = Vec::new();
        res.reserve(amounts.len());

        let mut upper_left = XY::new(0, 0);

        for s in amounts.iter() {
            let new_size: XY = if self.split_direction == SplitDirection::Vertical {
                (size.x, *s as u16).into()
            } else {
                (*s as u16, size.y).into()
            };

            res.push(Rect::new(upper_left, new_size));

            upper_left = upper_left
                + if self.split_direction == SplitDirection::Vertical {
                    XY::new(0, *s as u16)
                } else {
                    XY::new(*s as u16, 0).into()
                };
        }
        Some(res)
    }
}

impl<W: Widget> Layout<W> for SplitLayout<W> {
    fn get_focused<'a>(&self, parent: &'a W) -> &'a dyn Widget {
        self.children[self.focused].layout.get_focused(parent)
    }

    fn get_focused_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget {
        self.children[self.focused].layout.get_focused_mut(parent)
    }

    fn update_focus(&mut self, focus_update: FocusUpdate) -> bool {
        //TODO
        self.focused = self.focused % self.children.len();
        true
    }

    fn min_size(&self, owner: &W) -> XY {}

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
                child.layout.render(owner, focused_id, output);
            } else {
                debug!(
                    "skipping drawing split_layout item {:} {:?} beause it's outside view {:?}",
                    idx, rect, visible_rect
                );
            }
        }
    }
}
