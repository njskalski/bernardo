use std::cmp::max;

use log::{error, warn};

use crate::layout::layout::{Layout, WidgetWithRect};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SplitRule {
    Fixed(usize),
    MinSize,
    Proportional(f32),
}

struct SplitLayoutChild<W: Widget> {
    layout: Box<dyn Layout<W>>,
    split_rule: SplitRule,
}

pub struct SplitLayout<W: Widget> {
    children: Vec<SplitLayoutChild<W>>,
    split_direction: SplitDirection,
}

impl<W: Widget> SplitLayout<W> {
    pub fn new(split_direction: SplitDirection) -> Self {
        SplitLayout {
            children: vec![],
            split_direction,
        }
    }

    pub fn with(self, split_rule: SplitRule, child: Box<dyn Layout<W>>) -> Self {
        let mut children = self.children;
        let child = SplitLayoutChild {
            layout: child,
            split_rule,
        };

        children.push(child);

        SplitLayout { children, ..self }
    }

    fn get_just_rects(&self, size: XY, root: &W) -> Option<Vec<Rect>> {
        let free_axis = if self.split_direction == SplitDirection::Vertical {
            size.y as usize
        } else {
            size.x as usize
        };

        let fixed_amount = self.children.iter().fold(0, |acc, item| {
            acc + match item.split_rule {
                SplitRule::Fixed(i) => i,
                SplitRule::MinSize => {
                    let ms = item.layout.min_size(root);
                    if self.split_direction == SplitDirection::Vertical {
                        ms.y as usize
                    } else {
                        ms.x as usize
                    }
                }
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
                SplitRule::MinSize => {
                    let ms = child.layout.min_size(root);
                    let f = if self.split_direction == SplitDirection::Vertical {
                        ms.y
                    } else {
                        ms.x
                    } as usize;
                    amounts[idx] = f;
                }
                SplitRule::Proportional(prop) => {
                    sum_props += prop;
                }
            }
        }

        let unit = leftover as f32 / sum_props;
        let mut biggest_idx = 0;

        for (idx, child) in self.children.iter().enumerate() {
            if let SplitRule::Proportional(p) = child.split_rule {
                amounts[idx] = (unit * p) as usize;

                if idx > 0 {
                    if amounts[idx] > amounts[biggest_idx] {
                        biggest_idx = idx;
                    }
                }
            }
        }

        let sum_ = amounts.iter().fold(0 as usize, |a, b| a + *b);
        let difference = free_axis - sum_;
        amounts[biggest_idx] += difference;

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

        // debug!("split {:?} size {} rects {:?}", self.split_direction, size, res);

        debug_assert!(res.len() == self.children.len());

        Some(res)
    }
}

impl<W: Widget> Layout<W> for SplitLayout<W> {
    fn min_size(&self, root: &W) -> XY {
        let mut res = XY::new(0, 0);

        for child in self.children.iter() {
            let min_size = child.layout.min_size(root);
            match child.split_rule {
                SplitRule::Fixed(iusize) => {
                    if iusize > u16::MAX as usize {
                        error!("found too big SplitRule::Fixed to process, ignoring");
                        continue;
                    }

                    let i = iusize as u16;
                    if self.split_direction == SplitDirection::Vertical {
                        if min_size.y > i {
                            warn!("SplitRule::Fixed limits y below min_size.y");
                        }
                        res.x = max(res.x, min_size.x);
                        res.y += min_size.y;
                    } else {
                        if min_size.x > i {
                            warn!("SplitRule::Fixed limits x below min_size.x");
                        }

                        res.x += min_size.x;
                        res.y = max(res.y, min_size.y);
                    }
                }
                SplitRule::Proportional(_) | SplitRule::MinSize => {
                    if self.split_direction == SplitDirection::Vertical {
                        res.x = max(res.x, min_size.x);
                        res.y += min_size.y;
                    } else {
                        res.x += min_size.x;
                        res.y = max(res.y, min_size.y);
                    }
                }
            };
        }

        res
    }

    fn layout(&self, root: &mut W, output_size: XY) -> Vec<WidgetWithRect<W>> {
        let rects_op = self.get_just_rects(output_size, root);
        if rects_op.is_none() {
            warn!(
                "not enough space to get_rects split_layout: {:?}",
                output_size
            );
            return Vec::default();
        };

        let rects = rects_op.unwrap();
        let mut res: Vec<WidgetWithRect<W>> = vec![];

        debug_assert!(rects.len() == self.children.len());

        for (idx, child_layout) in self.children.iter().enumerate() {
            let rect = &rects[idx];
            let wirs = child_layout.layout.layout(root, rect.size);

            // debug!("A{} output_size {} parent {} children {:?}", wirs.len(), output_size, rect, wirs);
            //TODO add intersection checks

            for wir in wirs.into_iter() {
                let new_wir = wir.shifted(rect.pos);

                // debug!("output_size {} parent {} child {} res {}", output_size, rect, wir.rect, new_rect);
                debug_assert!(output_size.x >= new_wir.rect().lower_right().x);
                debug_assert!(output_size.y >= new_wir.rect().lower_right().y);

                res.push(new_wir);
            }
        }

        res
    }
}

#[cfg(test)]
pub mod tests {}