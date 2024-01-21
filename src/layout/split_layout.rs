use std::cmp::max;

use log::{debug, error, warn};

use crate::experiments::screenspace::Screenspace;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

/* TODO
One of many issues this file have is it's readability, starting with the fact that it seems that
SplitDirection type is non-intuitive. I added some drawings (that I had to reverse engineer from
code myself), then I'll add some tests, and then I can refactor.
 */

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SplitDirection {
    /*
    ┌───┬───┬───┐
    │ l │ l │ l │
    │ a │ a │ a │
    │ y │ y │ y │
    │ o │ o │ o │
    │ u │ u │ u │
    │ t │ t │ t │
    │   │   │   │
    │ 1 │ 2 │ 3 │
    └───┴───┴───┘
     */
    Horizontal,

    /*
    ┌────────────────────┐
    │ layout 1           │
    ├────────────────────┤
    │ layout 2           │
    ├────────────────────┤
    │ layout 3           │
    └────────────────────┘
     */
    Vertical,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SplitRule {
    // Uses exactly usize space on free axis
    Fixed(u16),
    /*
    I don't know how to implement "exact size", given the fact "exact_size" on layout needs to know
    "but how much space is available?". Well, that's what I am trying to figure out here.
    */
    // ExactSize,

    // Splits the free space proportionally to given numbers.
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
        let child = SplitLayoutChild { layout: child, split_rule };

        children.push(child);

        SplitLayout { children, ..self }
    }

    fn simple_layout(&self, root: &mut W, screenspace: Screenspace) -> LayoutResult<W> {
        let rects_op = self.get_just_rects(screenspace.output_size(), root);
        if rects_op.is_none() {
            warn!("not enough space to get_rects split_layout: {:?}", screenspace);
            return LayoutResult::new(Vec::default(), screenspace.output_size());
        };

        let rects = rects_op.unwrap();
        debug!("rects : {:?}", &rects);

        let mut res: Vec<WidgetWithRect<W>> = vec![];

        debug_assert!(rects.len() == self.children.len());

        for (idx, child_layout) in self.children.iter().enumerate() {
            let rect = &rects[idx];
            if let Some(visible_rect) = screenspace.visible_rect().intersect(*rect) {
                let mut visible_rect_in_child_space = visible_rect;
                visible_rect_in_child_space.pos -= rect.pos;

                let resp = child_layout
                    .layout
                    .layout(root, Screenspace::new(rect.size, visible_rect_in_child_space));

                for wir in resp.wwrs.into_iter() {
                    let new_wir = wir.shifted(rect.pos);

                    debug_assert!(screenspace.output_size().x >= new_wir.rect().lower_right().x);
                    debug_assert!(screenspace.output_size().y >= new_wir.rect().lower_right().y);

                    res.push(new_wir);
                }
            } else {
                debug!("skipping invisible layout #{} rect {:?}", idx, screenspace);
                continue;
            }
        }

        LayoutResult::new(res, screenspace.output_size())
    }

    fn get_just_rects(&self, output_size: XY, _root: &W) -> Option<Vec<Rect>> {
        let free_axis = if self.split_direction == SplitDirection::Vertical {
            output_size.y as usize
        } else {
            output_size.x as usize
        };

        let fixed_amounts_sum: usize = self.children.iter().fold(0, |acc, item| {
            acc + match item.split_rule {
                SplitRule::Fixed(i) => i as usize,
                // SplitRule::ExactSize => { 0 }
                SplitRule::Proportional(_) => 0,
            }
        });

        if fixed_amounts_sum > free_axis {
            return None;
        }

        let leftover = free_axis - fixed_amounts_sum;

        let mut amounts: Vec<usize> = vec![0; self.children.len()];
        let mut sum_props = 0.0f32;

        for (idx, child) in self.children.iter().enumerate() {
            match child.split_rule {
                SplitRule::Fixed(f) => {
                    amounts[idx] = f as usize;
                }
                // SplitRule::ExactSize => {
                //     let ms = child.layout.exact_size(root, output_size);
                //     let f = if self.split_direction == SplitDirection::Vertical {
                //         ms.y
                //     } else {
                //         ms.x
                //     } as usize;
                //     amounts[idx] = f;
                // }
                SplitRule::Proportional(prop) => {
                    sum_props += prop;
                }
            }
        }

        let mut biggest_prop_idx: Option<usize> = None;
        let unit = if sum_props > 0.0f32 { leftover as f32 / sum_props } else { 0.0f32 };

        for (idx, child) in self.children.iter().enumerate() {
            if let SplitRule::Proportional(p) = child.split_rule {
                if unit > 0.0f32 {
                    amounts[idx] += (unit * p) as usize;
                }

                // TODO this can potentially lead to extending a fixed-sized #0 slot
                if let Some(old_idx) = biggest_prop_idx {
                    if amounts[idx] > amounts[old_idx] {
                        biggest_prop_idx = Some(idx)
                    }
                } else {
                    biggest_prop_idx = Some(idx);
                }
            }
        }

        if let Some(idx) = biggest_prop_idx {
            let sum_ = amounts.iter().fold(0 as usize, |a, b| a + *b);
            let difference = free_axis - sum_;
            amounts[idx] += difference;
        }

        let mut res: Vec<Rect> = Vec::new();
        res.reserve(amounts.len());

        let mut upper_left = XY::new(0, 0);

        for s in amounts.iter() {
            let new_size: XY = if self.split_direction == SplitDirection::Vertical {
                (output_size.x, *s as u16).into()
            } else {
                (*s as u16, output_size.y).into()
            };

            res.push(Rect::new(upper_left, new_size));

            upper_left = upper_left
                + if self.split_direction == SplitDirection::Vertical {
                    XY::new(0, *s as u16)
                } else {
                    XY::new(*s as u16, 0).into()
                };
        }

        debug!("split {:?} size {} rects {:?}", self.split_direction, output_size, res);

        debug_assert!(res.len() == self.children.len());

        Some(res)
    }
}

impl<W: Widget> Layout<W> for SplitLayout<W> {
    fn prelayout(&self, root: &mut W) {
        for child in self.children.iter() {
            child.layout.prelayout(root);
        }
    }

    fn exact_size(&self, root: &W, output_size: XY) -> XY {
        let mut res = XY::new(0, 0);

        for child in self.children.iter() {
            let exact_size = child.layout.exact_size(root, output_size);
            match child.split_rule {
                SplitRule::Fixed(iusize) => {
                    let i = iusize as u16;
                    if self.split_direction == SplitDirection::Vertical {
                        if exact_size.y > i {
                            error!("SplitRule::Fixed limits y below exact_size.y");
                        }
                        res.x = max(res.x, exact_size.x);
                        res.y += exact_size.y;
                    } else {
                        if exact_size.x > i {
                            error!("SplitRule::Fixed limits x below exact_size.x");
                        }

                        res.x += exact_size.x;
                        res.y = max(res.y, exact_size.y);
                    }
                }
                SplitRule::Proportional(_) /*| SplitRule::ExactSize*/ => {
                    if self.split_direction == SplitDirection::Vertical {
                        res.x = max(res.x, exact_size.x);
                        res.y += exact_size.y;
                    } else {
                        res.x += exact_size.x;
                        res.y = max(res.y, exact_size.y);
                    }
                }
            };
        }

        res
    }

    fn layout(&self, root: &mut W, screenspace: Screenspace) -> LayoutResult<W> {
        self.simple_layout(root, screenspace)
    }
}
