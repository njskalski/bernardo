use std::cmp::max;

use log::warn;

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::io::output::Output;
use crate::primitives::arrow::Arrow;
use crate::primitives::scroll_enum::ScrollEnum;
use crate::primitives::size_constraint::{AlignEnum, SizeConstraint};
use crate::primitives::xy::XY;
use crate::widget::action_trigger::ActionTrigger;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::big_list::msg::BigListWidgetMsg;
use crate::widgets::list_widget::list_widget::ListWidgetMsg;

/*
This is list of bigger items, to be paired witch scroll.
 */

struct BigList<T: Widget> {
    //TODO I did not add the direction
    wid: WID,
    items: Vec<T>,
    item_idx: usize,

    last_size: Option<XY>,
}

impl<T: Widget> BigList<T> {
    pub const TYPENAME: &'static str = "big_list";

    pub fn new(items: Vec<T>) -> Self {
        BigList {
            wid: get_new_widget_id(),
            items,
            item_idx: 0,
            last_size: None,
        }
    }

    fn will_accept(&self, se: &ScrollEnum) -> bool {
        let can_go_up = self.item_idx > 0;
        let can_go_down = self.item_idx + 1 < self.items.len();

        match se {
            ScrollEnum::Arrow(arrow) => {
                match arrow {
                    Arrow::Up => can_go_up,
                    Arrow::Down => can_go_down,
                    Arrow::Left => false,
                    Arrow::Right => false,
                }
            }
            ScrollEnum::Home => can_go_up,
            ScrollEnum::End => can_go_down,
            ScrollEnum::PageUp => can_go_up,
            ScrollEnum::PageDown => can_go_down,
        }
    }

    fn last_page_height(&self) -> Option<u16> {
        self.last_size.map(|xy| xy.y)
    }
}

impl<T: Widget> Widget for BigList<T> {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn min_size(&self) -> XY {
        let mut xy = XY::new(10, 4); // TODO completely arbitrary

        for i in self.items.iter() {
            let c = i.min_size();
            xy.x = max(xy.x, c.x);
            xy.y = max(xy.y, c.y);
        }

        xy
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        self.last_size = Some(sc.visible_hint().size);

        // let mut pos_y: u16 = 0;
        // for item in self.items.iter_mut() {
        //     if let Some(max_y) = sc.y() {
        //         SizeConstraint(max_y)
        //     } else {
        //         item.update_and_layout()
        //     }
        // }
        todo!()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        match input_event {
            InputEvent::KeyInput(key) => {
                if let Some(se) = ScrollEnum::from_key(key) {
                    if self.will_accept(&se) {
                        BigListWidgetMsg::Scroll(se).someboxed()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<BigListWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd ListWidgetMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            BigListWidgetMsg::Scroll(se) => {
                match se {
                    ScrollEnum::Arrow(arrow) => {
                        match arrow {
                            Arrow::Up => {
                                if self.item_idx > 0 {
                                    self.item_idx -= 1;
                                } else {
                                    warn!("arrow up widget can't handle");
                                }
                                None
                            }
                            Arrow::Down => {
                                if self.item_idx + 1 < self.items.len() {
                                    self.item_idx += 1
                                } else {
                                    warn!("arrow down widget can't handle");
                                }
                                None
                            }
                            _ => None,
                        }
                    }
                    ScrollEnum::Home => {
                        if self.item_idx > 0 {
                            self.item_idx = 0;
                        } else {
                            warn!("home widget can't handle");
                        }
                        None
                    }
                    ScrollEnum::End => {
                        if self.item_idx + 1 < self.items.len() {
                            self.item_idx = self.items.len() - 1;
                        } else {
                            warn!("end widget can't handle");
                        }
                        None
                    }
                    ScrollEnum::PageUp => {
                        if let Some(height) = self.last_page_height() {
                            if self.item_idx > 0 {
                                // if self.pos < height {
                                //     self.pos = 0
                                // }
                            } else {
                                warn!("page_up widget can't handle")
                            }
                        } else {
                            warn!("page_up prior layout")
                        }
                        None
                    }
                    ScrollEnum::PageDown => {
                        if let Some(height) = self.last_page_height() {} else {
                            warn!("page_down prior layout")
                        }
                        None
                    }
                }
            }
        };
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        todo!()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }

    fn anchor(&self) -> XY {
        todo!()
    }
}