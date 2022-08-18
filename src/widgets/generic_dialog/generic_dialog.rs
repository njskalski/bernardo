use core::option::Option;
use std::fmt::Debug;

use log::{debug, error, warn};

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, subwidget, Theme, Widget, ZERO};
use crate::experiments::deref_str::DerefStr;
use crate::experiments::focus_group::FocusUpdate;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::keys::Key;
use crate::layout::display_state::GenericDisplayState;
use crate::layout::frame_layout::FrameLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::border::BorderStyle;
use crate::primitives::xy::XY;
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::button::ButtonWidget;
use crate::widgets::text_widget::TextWidget;

// TODO handle too small displays

pub trait KeyToMsg: Fn(&Key) -> Option<Box<dyn AnyMsg>> {}

const DEFAULT_INTERVAL: u16 = 2;
const CANCEL_LABEL: &'static str = "Cancel";

pub struct GenericDialog {
    wid: WID,

    display_state: Option<GenericDisplayState>,

    text_widget: TextWidget,

    with_border: Option<&'static BorderStyle>,

    buttons: Vec<ButtonWidget>,
    keystroke: Option<Box<dyn KeyToMsg>>,

}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GenericDialogMsg {
    FocusUpdate(FocusUpdate)
}

impl AnyMsg for GenericDialogMsg {}

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
        match &self.display_state {
            None => {
                error!("get_selected without display_state");
                0
            }
            Some(ds) => {
                let wid = ds.focus_group.get_focused();

                for (idx, button) in self.buttons.iter().enumerate() {
                    if button.id() == wid {
                        return idx;
                    }
                }

                error!("get_selected: focus_group returns WID that is not found among buttons : {}", wid);
                0
            }
        }
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

    pub fn with_keystroke(self, keystroke: Box<dyn KeyToMsg>) -> Self {
        Self {
            keystroke: Some(keystroke),
            ..self
        }
    }

    pub fn set_keystroke(&mut self, keystroke: Box<dyn KeyToMsg>) {
        self.keystroke = Some(keystroke);
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
        self.complex_layout(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("generic_dialog.on_input {:?}", input_event);

        return match input_event {
            InputEvent::FocusUpdate(focus_update) => {
                let can_update = self.display_state.as_ref().map(|ds| {
                    ds.focus_group.can_update_focus(focus_update)
                }).unwrap_or(false);

                if can_update {
                    Some(Box::new(GenericDialogMsg::FocusUpdate(focus_update)))
                } else {
                    None
                }
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("generic_dialog.update {:?}", msg);

        let our_msg = msg.as_msg::<GenericDialogMsg>();
        if our_msg.is_none() {
            // this case makes sense here, we just pass the messages from buttons through.
            // no warning.
            debug!("generic_dialog passes through message {:?} to parent", msg);
            return Some(msg);
        }

        return match our_msg.unwrap() {
            GenericDialogMsg::FocusUpdate(focus_update) => {
                // warn!("updating focus");
                self.display_state.as_mut().map(
                    |ds| {
                        let _msg = ds.focus_group.update_focus(*focus_update);
                        // warn!("focus updated {}", msg);
                        None
                    }
                ).unwrap_or_else(|| {
                    error!("failed retrieving display_state");
                    None
                })
            }
            unknown_msg => {
                warn!("GenericDialog.update : unknown message {:?}", unknown_msg);
                None
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.complex_render(theme, focused, output)
    }
}

impl ComplexWidget for GenericDialog {
    fn internal_layout(&self, _size: XY) -> Box<dyn Layout<Self>> {
        let text_layout = LeafLayout::new(subwidget!(Self.text_widget)).boxed();

        // let mut button_layout = SplitLayout::new(SplitDirection::Vertical);
        let button_layouts: Vec<Box<dyn Layout<Self>>> = (0..self.buttons.len()).map(|idx| {
            let idx1 = idx;
            let idx2 = idx;
            LeafLayout::new(SubwidgetPointer::new(
                Box::new(move |s: &Self| {
                    &s.buttons[idx1]
                }),
                Box::new(move |s: &mut Self| {
                    &mut s.buttons[idx2]
                }),
            )).boxed()
        }).collect();

        let button_layout = button_layouts.into_iter().fold(
            SplitLayout::new(SplitDirection::Horizontal),
            |acc, layout| {
                acc.with(SplitRule::Proportional(1.0), layout)
            })
            .boxed();

        let total_layout = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0),
                  text_layout)
            .with(SplitRule::Fixed(1), button_layout)
            .boxed();

        let frame_layout = FrameLayout::new(total_layout, XY::new(2, 2)).boxed();

        frame_layout
    }

    fn get_default_focused(&self) -> SubwidgetPointer<GenericDialog> {
        todo!()
    }

    fn set_display_state(&mut self, ds: DisplayState<GenericDialog>) {
        todo!()
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<GenericDialog>> {
        todo!()
    }
}