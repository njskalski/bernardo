use core::option::Option;
use std::borrow::Borrow;
use std::f32::consts::E;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::rc::Rc;

use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Theme, Widget, ZERO};
use crate::experiments::deref_str::DerefStr;
use crate::io::keys::Key;
use crate::io::sub_output::SubOutput;
use crate::layout::display_state::DisplayState;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::empty_layout::EmptyLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::border::BorderStyle;
use crate::primitives::helpers::fill_output;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID, WidgetAction};
use crate::widgets::button::ButtonWidget;
use crate::widgets::text_widget::TextWidget;

pub trait KeyToMsg: Fn(&Key) -> Option<Box<dyn AnyMsg>> {}

const DEFAULT_INTERVAL: u16 = 2;
const CANCEL_LABEL: &'static str = "Cancel";

pub struct GenericDialog {
    wid: WID,

    display_state: Option<DisplayState>,

    text_widget: TextWidget,

    with_border: Option<&'static BorderStyle>,

    buttons: Vec<ButtonWidget>,
    keystroke: Option<Box<dyn KeyToMsg>>,

}

impl GenericDialog {
    pub fn new(text: Box<dyn DerefStr>) -> Self {
        Self {
            wid: get_new_widget_id(),
            display_state: None,
            text_widget: TextWidget::new(text),
            with_border: None,
            buttons: vec![],
            keystroke: None,
        }
    }

    pub fn with_option(self, button: ButtonWidget) -> Self {
        let mut buttons = self.buttons;
        buttons.push(button);
        Self {
            buttons,
            ..self
        }
    }

    pub fn add_option(&mut self, button: ButtonWidget) {
        self.buttons.push(button);
    }

    pub fn get_options(&self) -> &Vec<ButtonWidget> {
        &self.buttons
    }

    pub fn get_options_mut(&mut self) -> &mut Vec<ButtonWidget> {
        &mut self.buttons
    }

    pub fn get_selected(&self) -> usize {
        todo!()
    }

    pub fn with_border(self, border_style: &'static BorderStyle) -> Self {
        Self {
            with_border: Some(border_style),
            ..self
        }
    }


    pub fn get_total_options_width(&self, interval: u16) -> u16 {
        let mut result: usize = 0;
        for (idx, button) in self.buttons.iter().enumerate() {
            result += button.min_size().x as usize;
            if idx + 1 < self.buttons.len() {
                result += interval as usize;
            }
        }
        if result > u16::MAX as usize {
            error!("absourdly long options_width, returning u16::MAX");
            u16::MAX
        } else {
            result as u16
        }
    }

    pub fn with_keystroke(self, keystroke: Box<KeyToMsg>) -> Self {
        Self {
            keystroke: Some(keystroke),
            ..self
        }
    }

    pub fn set_keystroke(&mut self, keystroke: Box<KeyToMsg>) {
        self.keystroke = Some(keystroke);
    }

    fn internal_layout(&mut self, size: XY) -> Vec<WidgetIdRect> {
        let mut text_layout = LeafLayout::new(&mut self.text_widget);

        // let mut button_layout = SplitLayout::new(SplitDirection::Vertical);
        let mut button_layouts: Vec<LeafLayout> = self.buttons.iter_mut().map(
            |but| {
                LeafLayout::new(but as &mut dyn Widget)
            }).collect();

        let mut button_layout = button_layouts.iter_mut().fold(
            SplitLayout::new(SplitDirection::Vertical),
            |acc, layout| {
                acc.with(SplitRule::Proportional(1.0), layout)
            });

        let mut total_layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0),
                  &mut text_layout as &mut dyn Layout)
            .with(SplitRule::Fixed(1), &mut button_layout as &mut dyn Layout);

        total_layout.calc_sizes(size)
    }
}

impl Widget for GenericDialog {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "GenericDialog"
    }

    fn min_size(&self) -> XY {
        let mut total_size = self.text_widget.text_size();

        if !self.buttons.is_empty() {
            let op_widths = self.get_total_options_width(DEFAULT_INTERVAL);

            total_size.y += 2;
            if total_size.x < op_widths {
                total_size.x = op_widths;
            }
        }

        total_size + if self.with_border.is_some() { XY::new(2, 2) } else { ZERO }
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        let size = sc.hint().size;
        let wirs = self.internal_layout(size);

        let focus_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        self.display_state = Some(DisplayState::new(size, wirs));

        // re-setting focus.
        match (focus_op, &mut self.display_state) {
            (Some(focus), Some(ds)) => { ds.focus_group.set_focused(focus); },
            _ => {}
        };

        size
    }

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        Some(msg)
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.ui.non_focused.background, output);

        match self.display_state.borrow().as_ref() {
            None => warn!("failed rendering GenericDialog without cached_sizes"),
            Some(cached_sizes) => {
                for wir in &cached_sizes.widget_sizes {
                    match self.get_subwidget(wir.wid) {
                        Some(widget) => {
                            let sub_output = &mut SubOutput::new(output, wir.rect);
                            widget.render(theme,
                                          focused && cached_sizes.focus_group.get_focused() == widget.id(),
                                          sub_output,
                            );
                        },
                        None => {
                            warn!("subwidget {} not found!", wir.wid);
                        }
                    }
                }
            }
        }
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(|wid| self.get_subwidget(wid)).flatten()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(move |wid| self.get_subwidget_mut(wid)).flatten()
    }

    fn subwidgets_mut(&mut self) -> Box<dyn std::iter::Iterator<Item=&mut dyn Widget> + '_> {
        debug!("call to generic_dialog subwidget_mut on {}", self.id());
        Box::new(self.buttons.iter_mut().map(|button| {
            button as &mut dyn Widget
        }).chain(
            iter::once(&mut self.text_widget as &mut dyn Widget)
        ))
    }

    fn subwidgets(&self) -> Box<dyn std::iter::Iterator<Item=&dyn Widget> + '_> {
        debug!("call to generic_dialog subwidget on {}", self.id());
        Box::new(self.buttons.iter().map(|button| {
            button as &dyn Widget
        }).chain(
            iter::once(&self.text_widget as &dyn Widget)
        ))
    }
}

