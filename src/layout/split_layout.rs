use std::cmp::max;

use log::{debug, error, warn};

use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::widget_with_rect::WidgetWithRect;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
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
    // Uses exactly min_size of sublayout
    MinSize,

    // In case where free axis is constrained, splits the free space proportionally to given numbers.
    // In case where free axis in unconstrained, allows sublayouts unconstrained expansion, but
    //  does not expand them further to meet the proportion (argument ignored).
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

    fn simple_layout(&self, root: &mut W, output_size: XY, sc: SizeConstraint) -> LayoutResult<W> {
        let rects_op = self.get_just_rects(output_size, root);
        if rects_op.is_none() {
            warn!(
                "not enough space to get_rects split_layout: {:?}",
                output_size
            );
            return LayoutResult::new(Vec::default(), output_size);
        };

        let rects = rects_op.unwrap();
        // debug!("rects : {:?}", &rects);

        let mut res: Vec<WidgetWithRect<W>> = vec![];

        debug_assert!(rects.len() == self.children.len());

        for (idx, child_layout) in self.children.iter().enumerate() {
            let rect = &rects[idx];
            let new_sc = match sc.cut_out_rect(*rect) {
                Some(new_sc) => new_sc,
                None => {
                    debug!("skipping invisible layout #{} rect {} sc {}", idx, rect, sc);
                    continue;
                }
            };
            let resp = child_layout.layout.layout(root, new_sc);

            for wir in resp.wwrs.into_iter() {
                let new_wir = wir.shifted(rect.pos);

                debug_assert!(output_size.x >= new_wir.rect().lower_right().x);
                debug_assert!(output_size.y >= new_wir.rect().lower_right().y);

                res.push(new_wir);
            }
        }

        LayoutResult::new(res, output_size)
    }

    /*
    In this variant, we give all "Proportional" children "as much space as they want" - every
    proportion of infinity is infinite.

    Surprisingly, implementation of this variant is simpler, mostly because there is no "proportionality".
     */
    fn complicated_layout(&self, root: &mut W, sc: SizeConstraint) -> LayoutResult<W> {
        let mut result: Vec<WidgetWithRect<W>> = Vec::new();
        let mut offset = XY::ZERO;

        let non_free_axis = if self.split_direction == SplitDirection::Vertical {
            sc.y().unwrap_or_else(|| {
                error!("fake unwrap, returning safe default");
                10 // TODO
            })
        } else {
            sc.x().unwrap_or_else(|| {
                error!("fake unwrap, returning safe default");
                10 //TODO
            })
        };

        for (child_idx, child) in self.children.iter().enumerate() {
            let min_child_size = child.layout.min_size(root);

            /*
            Ok, here things get complex. I can skip layouting *below* viewport. I cannot skip
            layouting *above* it, because I woudln't know the offset.

            Now calculating offset is possible without layout in cases of Fixed and MinSize rules.
            In case of Proportional *I have to delegate* layouting down the tree, hoping for early
            exits from subsequent layouts.
             */

            match child.split_rule {
                SplitRule::Fixed(fixed) => {
                    let local_size = if self.split_direction == SplitDirection::Vertical {
                        XY::new(non_free_axis, fixed)
                    } else {
                        XY::new(fixed, non_free_axis)
                    };

                    let rect = Rect::new(offset, local_size);

                    if min_child_size <= rect.size {
                        match sc.cut_out_rect(rect) {
                            Some(new_sc) => {
                                let resp = child.layout.layout(root, new_sc);
                                for wwrsc in resp.wwrs.into_iter() {
                                    result.push(wwrsc.shifted(rect.pos))
                                };
                            }
                            None => {
                                debug!("skipping layouting FixedSize child #{} because rect is invisible", child_idx);
                                // no continue, just let it go to offset calculation below.
                            }
                        };
                    } else {
                        debug!("skipping child #{} because rect is too small", child_idx);
                    }

                    if self.split_direction == SplitDirection::Vertical {
                        offset += XY::new(0, fixed);
                    } else {
                        offset += XY::new(fixed, 0);
                    }
                }
                SplitRule::MinSize => {
                    let local_size = if self.split_direction == SplitDirection::Vertical {
                        XY::new(non_free_axis, min_child_size.y)
                    } else {
                        XY::new(min_child_size.x, non_free_axis)
                    };

                    let rect = Rect::new(offset, local_size);

                    if min_child_size <= rect.size {
                        match sc.cut_out_rect(rect) {
                            Some(new_sc) => {
                                let resp = child.layout.layout(root, new_sc);
                                for wwrsc in resp.wwrs.into_iter() {
                                    result.push(wwrsc.shifted(rect.pos))
                                };
                            }
                            None => {
                                debug!("skipping layouting MinSize child #{} because rect is invisible", child_idx);
                                // no continue, let it flow to offset calculation below
                            }
                        };
                    } else {
                        debug!("skipping child #{} because rect is too small", child_idx);
                    }

                    if self.split_direction == SplitDirection::Vertical {
                        offset += XY::new(0, min_child_size.y);
                    } else {
                        offset += XY::new(min_child_size.x, 0);
                    }
                }
                SplitRule::Proportional(_) => {
                    let new_sc = match sc.substract(offset) {
                        Some(sc) => sc,
                        None => {
                            debug!("not layouting child #{}, cut_out_margin => None", child_idx);
                            /*
                            So I can't skip layouting here, because I would loose offset above viewport.
                            Now I have following options:
                            1) create a degenerated SC, with empty viewport. No invariants violated.
                            2) make viewport optional (allow no-viewport sc's), but that's a change in 200 places.
                            3) [impossible] add "max_size" to layout trait. But where would this information come from?
                            yeah, options 1 and 2 are the only valid ones, second being typesafe too.

                             */

                            continue;
                        }
                    };

                    let resp = child.layout.layout(root, new_sc);
                    for wwrsc in resp.wwrs.into_iter() {
                        let item = wwrsc.shifted(offset);
                        result.push(item);
                    };

                    if self.split_direction == SplitDirection::Vertical {
                        offset += XY::new(0, resp.total_size.y);
                    } else {
                        offset += XY::new(resp.total_size.x, 0);
                    }
                }
            };
        }

        LayoutResult::new(result, offset)
    }

    fn get_just_rects(&self, size: XY, root: &W) -> Option<Vec<Rect>> {
        let free_axis = if self.split_direction == SplitDirection::Vertical {
            size.y as usize
        } else {
            size.x as usize
        };

        let fixed_amount: usize = self.children.iter().fold(0, |acc, item| {
            acc + match item.split_rule {
                SplitRule::Fixed(i) => i as usize,
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
                    amounts[idx] = f as usize;
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

                // TODO this can potentially lead to extending a fixed-sized #0 slot
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

        debug!("split {:?} size {} rects {:?}", self.split_direction, size, res);

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

    fn layout(&self, root: &mut W, sc: SizeConstraint) -> LayoutResult<W> {
        if let Some(simple_size) = sc.as_finite() {
            self.simple_layout(root, simple_size, sc)
        } else {
            if self.split_direction == SplitDirection::Vertical && sc.x().is_none() {
                error!("messed up case, where we have a split direction on non-free axis.");
            }
            debug!("entered complicated layout");
            self.complicated_layout(root, sc)
        }
    }
}

/*
not returning a cut_out_rect that would have been invisible (rect
Rect { pos: XY { x: 0, y: 0 }, size: XY { x: 49, y: 1 } },

SizeConstraint { x: Some(73), y: Some(1), visible: Rect { pos: XY { x: 48, y: 36 }, size: XY { x: 25, y: 1 } } })

 */