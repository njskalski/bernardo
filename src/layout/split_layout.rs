use std::cmp::min;
use std::net::Shutdown::Read;
use std::slice::Iter;

use log::debug;
use log::warn;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::layout::{Layout, WidgetGetterMut, WidgetIdRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

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

struct SplitLayoutChild<'a> {
    layout: &'a mut dyn Layout,
    split_rule: SplitRule,
}

pub struct SplitLayout<'a> {
    children: Vec<SplitLayoutChild<'a>>,
    split_direction: SplitDirection,
}

impl<'a> SplitLayout<'a> {
    pub fn new(split_direction: SplitDirection) -> Self {
        SplitLayout {
            children: vec![],
            split_direction,
        }
    }

    pub fn with(self, split_rule: SplitRule, child: &'a mut dyn Layout) -> Self {
        let mut children = self.children;
        let child = SplitLayoutChild {
            layout: child,
            split_rule,
        };

        children.push(child);

        SplitLayout { children, ..self }
    }

    fn get_just_rects(&self, size: XY) -> Option<Vec<Rect>> {
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

        debug!("split {:?} size {} rects {:?}", self.split_direction, size, res);

        debug_assert!(res.len() == self.children.len());

        Some(res)
    }
}

impl<'a> Layout for SplitLayout<'a> {
    fn min_size(&self) -> XY {
        let mut minxy = XY::new(0, 0);

        for child in self.children.iter() {
            let min_size = child.layout.min_size();
            match child.split_rule {
                SplitRule::Fixed(iusize) => {
                    if iusize > u16::MAX as usize {
                        warn!("found too big SplitRule::Fixed to process");
                        continue;
                    }

                    let i = iusize as u16;
                    if self.split_direction == SplitDirection::Vertical {
                        if min_size.y > i {
                            warn!("SplitRule::Fixed limits y below min_size.y");
                        }

                        if minxy.y < i {
                            minxy.y = i;
                        }
                        if minxy.x < min_size.x {
                            minxy.x = min_size.x;
                        }
                    } else {
                        if min_size.x > i {
                            warn!("SplitRule::Fixed limits x below min_size.x");
                        }

                        if minxy.x < i {
                            minxy.x = i;
                        }
                        if minxy.y < min_size.y {
                            minxy.y = min_size.y;
                        }
                    }
                }
                SplitRule::Proportional(_) => {
                    if minxy.x < min_size.x {
                        minxy.x = min_size.x;
                    }
                    if minxy.y < min_size.y {
                        minxy.y = min_size.y;
                    }
                }
            };
        }

        minxy
    }

    fn calc_sizes(&mut self, output_size: XY) -> Vec<WidgetIdRect> {
        let rects_op = self.get_just_rects(output_size);
        if rects_op.is_none() {
            warn!(
                "not enough space to get_rects split_layout: {:?}",
                output_size
            );
            return vec![];
        };

        let rects = rects_op.unwrap();
        let mut res: Vec<WidgetIdRect> = vec![];

        debug_assert!(rects.len() == self.children.len());

        for (idx, child_layout) in self.children.iter_mut().enumerate() {
            let rect = &rects[idx];
            let wirs = child_layout.layout.calc_sizes(rect.size);

            debug!("A{} output_size {} parent {} children {:?}", wirs.len(), output_size, rect, wirs);
            //TODO add intersection checks

            for wir in wirs.iter() {
                let wid = wir.wid;
                let new_rect = wir.rect.shifted(rect.pos);

                debug!("output_size {} parent {} child {} res {}", output_size, rect, wir.rect, new_rect);
                debug_assert!(output_size.x >= new_rect.lower_right().x);
                debug_assert!(output_size.y >= new_rect.lower_right().y);

                res.push(WidgetIdRect {
                    wid,
                    rect: new_rect,
                });
            }
        }

        res
    }
}
