use std::cmp::max;

use log::warn;

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::io::output::Output;
use crate::primitives::scroll_enum::ScrollEnum;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::action_trigger::ActionTrigger;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::big_list::msg::BigListWidgetMsg;
use crate::widgets::list_widget::list_widget::ListWidgetMsg;

struct BigList<T: Widget> {
    //TODO I added not the direction
    wid: WID,
    items: Vec<T>,
    pos: usize,
}

impl<T: Widget> BigList<T> {
    pub const TYPENAME: &'static str = "big_list";

    pub fn new(items: Vec<T>) -> Self {
        BigList {
            wid: get_new_widget_id(),
            items,
            pos: 0,
        }
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
        todo!()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        match input_event {
            InputEvent::KeyInput(key) => {
                ScrollEnum::from_key(key).map(|se| {
                    BigListWidgetMsg::Scroll(se).someboxed()
                }).flatten()
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
                    ScrollEnum::Arrow(_) => {}
                    ScrollEnum::Home => {}
                    ScrollEnum::End => {}
                    ScrollEnum::PageUp => {}
                    ScrollEnum::PageDown => {}
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