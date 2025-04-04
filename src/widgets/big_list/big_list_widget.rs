use log::{debug, error, warn};

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::arrow::Arrow;
use crate::primitives::rect::Rect;
use crate::primitives::scroll_enum::ScrollEnum;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::big_list::msg::BigListWidgetMsg;
use crate::widgets::text_widget::TextWidget;

/*
This is list of bigger items, to be paired with scroll.
 */

//TODO implement pg-up pg-down

pub struct BigList<T: Widget> {
    //TODO I did not add the direction
    wid: WID,
    items: Vec<(u16, T)>,
    item_idx: usize,

    last_size: Option<XY>,

    no_items_text: TextWidget,

    display_state: Option<DisplayState<Self>>,
    kite: XY,

    size_policy: SizePolicy,
}

impl<T: Widget> BigList<T> {
    pub const TYPENAME: &'static str = "big_list";

    pub fn new(items: Vec<(u16, T)>) -> Self {
        BigList {
            wid: get_new_widget_id(),
            items,
            item_idx: 0,
            last_size: None,
            no_items_text: TextWidget::new(Box::new("empty")),
            display_state: None,
            kite: XY::ZERO,
            size_policy: SizePolicy::MATCH_LAYOUTS_WIDTH,
        }
    }

    fn will_accept(&self, se: &ScrollEnum) -> bool {
        let can_go_up = self.item_idx > 0;
        let can_go_down = self.item_idx + 1 < self.items.len();

        match se {
            ScrollEnum::Arrow(arrow) => match arrow {
                Arrow::Up => can_go_up,
                Arrow::Down => can_go_down,
                Arrow::Left => false,
                Arrow::Right => false,
            },
            ScrollEnum::Home => can_go_up,
            ScrollEnum::End => can_go_down,
            ScrollEnum::PageUp => can_go_up,
            ScrollEnum::PageDown => can_go_down,
        }
    }

    pub fn with_size_policy(self, size_policy: SizePolicy) -> Self {
        Self { size_policy, ..self }
    }

    fn last_page_height(&self) -> Option<u16> {
        self.last_size.map(|xy| xy.y)
    }

    fn get_item_widget_ptr(&self, idx: usize) -> SubwidgetPointer<Self> {
        let idx2 = idx;
        SubwidgetPointer::new(
            Box::new(move |s: &Self| &s.items[idx].1),
            Box::new(move |s: &mut Self| &mut s.items[idx2].1),
        )
    }

    pub fn add_item(&mut self, height: u16, item: T) {
        if self.items.is_empty() {
            self.update_focus_path()
        }

        self.items.push((height, item));
    }

    fn set_kite(&mut self, going_up: bool) {
        let mut y = 0 as u16;

        for i in self.items.iter().take(self.item_idx) {
            y += i.0;
        }

        if going_up == false {
            let last_height = self.items[self.item_idx].0;
            y += if last_height > 0 { last_height - 1 } else { 0 };
            // size points one character AFTER the avatar, and kite is inclusive.
        }

        self.kite = XY::new(0, y);
    }

    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|(_split_rule, widget)| widget)
    }

    pub fn is_empty(&self) -> bool {
        self.items().next().is_none()
    }

    fn update_focus_path(&mut self) {
        let widget_ptr = self.get_item_widget_ptr(self.item_idx);
        if let Some(ds) = self.display_state.as_mut() {
            ds.focused = widget_ptr;
        } else {
            warn!("no display_state");
        }
    }

    pub fn get_selected_id(&self) -> usize {
        self.item_idx
    }

    pub fn get_selected_item(&self) -> Option<&T> {
        self.items.get(self.item_idx).map(|(_, item)| item)
    }

    pub fn get_selected_item_mut(&mut self) -> Option<&mut T> {
        self.items.get_mut(self.item_idx).map(|(_, item)| item)
    }
}

impl<T: Widget> Widget for BigList<T> {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }
    fn prelayout(&mut self) {
        debug!("prelayout {}", self.typename());

        self.complex_prelayout();
    }

    fn full_size(&self) -> XY {
        let mut size = XY::new(20, 0); // TODO completely width

        for i in &self.items {
            size.y += i.0;
        }

        size
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn size_policy(&self) -> SizePolicy {
        self.size_policy
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
                    ScrollEnum::Arrow(arrow) => match arrow {
                        Arrow::Up => {
                            if self.item_idx > 0 {
                                self.item_idx -= 1;
                                self.update_focus_path();
                                self.set_kite(true);
                            } else {
                                warn!("arrow up widget can't handle");
                            }
                            None
                        }
                        Arrow::Down => {
                            if self.item_idx + 1 < self.items.len() {
                                self.item_idx += 1;
                                self.update_focus_path();
                                self.set_kite(false);
                            } else {
                                warn!("arrow down widget can't handle");
                            }
                            None
                        }
                        _ => None,
                    },
                    ScrollEnum::Home => {
                        if self.item_idx > 0 {
                            self.item_idx = 0;
                            self.update_focus_path();
                            self.set_kite(true);
                        } else {
                            warn!("home widget can't handle");
                        }
                        None
                    }
                    ScrollEnum::End => {
                        if self.item_idx + 1 < self.items.len() {
                            self.item_idx = self.items.len() - 1;
                            self.update_focus_path();
                            self.set_kite(false);
                        } else {
                            warn!("end widget can't handle");
                        }
                        None
                    }
                    ScrollEnum::PageUp => {
                        if let Some(_height) = self.last_page_height() {
                            if self.item_idx > 0 {
                                // if self.pos < height {
                                //     self.pos = 0
                                // } else {
                                //
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
                        if let Some(_height) = self.last_page_height() {
                        } else {
                            warn!("page_down prior layout")
                        }
                        None
                    }
                }
            }
        };
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.complex_get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.complex_get_focused_mut()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(any(test, feature = "fuzztest"))]
        {
            let total_size = self.display_state.as_ref().unwrap().total_size;
            output.emit_metadata(crate::io::output::Metadata {
                id: self.wid,
                typename: self.typename().to_string(),
                rect: Rect::from_zero(total_size),
                focused,
            });
        }

        self.complex_render(theme, focused, output)
    }

    fn kite(&self) -> XY {
        self.kite
    }
}

impl<T: Widget> ComplexWidget for BigList<T> {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        if self.items.is_empty() {
            LeafLayout::new(subwidget!(Self.no_items_text)).boxed()
        } else {
            let mut spl = SplitLayout::new(SplitDirection::Vertical);

            for idx in 0..self.items.len() {
                let height = self.items[idx].0;
                spl = spl.with(SplitRule::Fixed(height), LeafLayout::new(self.get_item_widget_ptr(idx)).boxed());
            }

            spl.boxed()
        }
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        if self.items.is_empty() {
            subwidget!(Self.no_items_text)
        } else {
            self.get_item_widget_ptr(0)
        }
    }

    fn set_display_state(&mut self, display_state: DisplayState<Self>) {
        self.display_state = Some(display_state);
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<Self>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}
