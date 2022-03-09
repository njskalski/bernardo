use core::option::Option;
use std::alloc::Layout;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Theme, Widget, ZERO};
use crate::experiments::deref_str::DerefStr;
use crate::io::keys::Key;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::layout::WidgetIdRect;
use crate::primitives::border::BorderStyle;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID, WidgetAction};
use crate::widgets::button::ButtonWidget;
use crate::widgets::generic_dialog::generic_dialog_msg::GenericDialogMsg;

pub trait KeyToMsg: Fn(&Key) -> Option<Box<dyn AnyMsg>> {}

const DEFAULT_INTERVAL: u16 = 2;
const CANCEL_LABEL: &'static str = "Cancel";

pub struct GenericDialog {
    wid: WID,
    text: Rc<String>,

    with_border: Option<&'static BorderStyle>,

    buttons: Vec<ButtonWidget>,
    keystroke: Option<Box<dyn KeyToMsg>>,
}

impl GenericDialog {
    pub fn new(text: Rc<String>) -> Self {
        let button: ButtonWidget = ButtonWidget::new(Box::new(CANCEL_LABEL))
            .with_on_hit(|_| GenericDialogMsg::Cancel.someboxed());

        Self {
            wid: get_new_widget_id(),
            text,
            with_border: None,
            buttons: vec![button],
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

    pub fn text_size(&self) -> XY {
        let mut size = ZERO;
        for (idx, line) in self.text.lines().enumerate() {
            size.x = size.x.max(line.width_cjk() as u16); // TODO
            size.y = idx as u16;
        }

        size
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

    fn internal_layout(&self, size: XY) -> Vec<WidgetIdRect> {
        let text = self.text_size();

        vec![]
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
        let mut total_size = self.text_size();

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
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::Esc => GenericDialogMsg::Cancel.someboxed(),
                    Keycode::ArrowLeft => GenericDialogMsg::Left.someboxed(),
                    Keycode::ArrowRight => GenericDialogMsg::Right.someboxed(),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("save_file_dialog.update {:?}", msg);

        let our_msg = msg.as_msg::<GenericDialogMsg>();
        if our_msg.is_none() {
            warn!("expecetd SaveFileDialogMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            _ => None,
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }
}

