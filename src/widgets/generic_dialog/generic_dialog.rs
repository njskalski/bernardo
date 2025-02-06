use core::option::Option;
use std::fmt::Debug;

use log::{debug, error, warn};

use crate::config::theme::Theme;
use crate::experiments::focus_group::FocusUpdate;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::io::output::Output;
use crate::layout::frame_layout::FrameLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::border::BorderStyle;
use crate::primitives::printable::Printable;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::button::ButtonWidget;
use crate::widgets::frame_widget::FrameWidget;
use crate::widgets::text_widget::TextWidget;
use crate::{subwidget, todosubwidgetunwrap};

// TODO handle too small displays

const DEFAULT_INTERVAL: u16 = 2;
const CANCEL_LABEL: &str = "Cancel";

pub struct GenericDialog {
    wid: WID,

    display_state: Option<DisplayState<GenericDialog>>,

    label_op: Option<String>,
    text_widget: TextWidget,

    border_op: Option<FrameWidget>,

    buttons: Vec<ButtonWidget>,
    keystroke: Option<Box<dyn Fn(Key) -> Option<Box<dyn AnyMsg>>>>,

    enable_arrows: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GenericDialogMsg {
    FocusUpdate(FocusUpdate),
}

impl AnyMsg for GenericDialogMsg {}

impl GenericDialog {
    pub const TYPENAME: &'static str = "generic_dialog";

    pub fn new(text: Box<dyn Printable>) -> Self {
        Self {
            wid: get_new_widget_id(),
            display_state: None,
            label_op: None,
            text_widget: TextWidget::new(text).with_size_policy(SizePolicy::MATCH_LAYOUT),
            border_op: None,
            buttons: vec![],
            keystroke: None,
            enable_arrows: false,
        }
    }

    pub fn with_arrows_as_focus_updates(self) -> Self {
        Self {
            enable_arrows: true,
            ..self
        }
    }

    pub fn with_label(self, label: String) -> Self {
        Self {
            label_op: Some(label),
            ..self
        }
    }

    pub fn with_option(self, button: ButtonWidget) -> Self {
        let mut buttons = self.buttons;
        buttons.push(button);
        Self { buttons, ..self }
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

    pub fn with_border(self, border_style: &'static BorderStyle, label_op: Option<String>) -> Self {
        Self {
            border_op: Some(FrameWidget::new(border_style.clone(), label_op)),
            ..self
        }
    }

    pub fn get_total_options_width(&self, interval: u16) -> u16 {
        let mut result: usize = 0;
        for (idx, button) in self.buttons.iter().enumerate() {
            result += button.full_size().x as usize;
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

    pub fn with_keystroke(self, keystroke: Box<dyn Fn(Key) -> Option<Box<dyn AnyMsg>>>) -> Self {
        Self {
            keystroke: Some(keystroke),
            ..self
        }
    }

    pub fn set_keystroke(&mut self, keystroke: Box<dyn Fn(Key) -> Option<Box<dyn AnyMsg>>>) {
        self.keystroke = Some(keystroke);
    }
}

impl Widget for GenericDialog {
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
        self.complex_prelayout();
    }

    fn full_size(&self) -> XY {
        let mut total_size = self.text_widget.text_size();

        if !self.buttons.is_empty() {
            let op_widths = self.get_total_options_width(DEFAULT_INTERVAL);

            total_size.y += 2;
            if total_size.x < op_widths {
                total_size.x = op_widths;
            }
        }

        total_size + if self.border_op.is_some() { XY::new(2, 2) } else { XY::ZERO }
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("generic_dialog.on_input {:?}", input_event);

        return match input_event {
            InputEvent::FocusUpdate(focus_update) => {
                if self.will_accept_focus_update(focus_update) {
                    Some(Box::new(GenericDialogMsg::FocusUpdate(focus_update)))
                } else {
                    None
                }
            }
            InputEvent::KeyInput(key) if self.enable_arrows && key.as_focus_update().is_some() => {
                let focus_update = key.as_focus_update().unwrap();
                if self.will_accept_focus_update(focus_update) {
                    Some(Box::new(GenericDialogMsg::FocusUpdate(focus_update)))
                } else {
                    None
                }
            }
            InputEvent::KeyInput(key) if self.keystroke.is_some() => {
                let ks = self.keystroke.as_ref().unwrap();
                ks(key)
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

        #[allow(unreachable_patterns)]
        return match our_msg.unwrap() {
            GenericDialogMsg::FocusUpdate(focus_update) => {
                self.update_focus(*focus_update);
                None
            }
            unknown_msg => {
                warn!("GenericDialog.update : unknown message {:?}", unknown_msg);
                None
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(any(test, feature = "fuzztest"))]
        {
            let size = crate::unpack_unit_e!(self.display_state.as_ref(), "render before layout",).total_size;

            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: Rect::from_zero(size),
                focused,
            });
        }

        self.complex_render(theme, focused, output)
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.complex_get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.complex_get_focused_mut()
    }
}

impl ComplexWidget for GenericDialog {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let text_layout = LeafLayout::new(subwidget!(Self.text_widget)).boxed();

        // let mut button_layout = SplitLayout::new(SplitDirection::Vertical);
        let button_layouts: Vec<Box<dyn Layout<Self>>> = (0..self.buttons.len())
            .map(|idx| {
                let idx1 = idx;
                let idx2 = idx;
                LeafLayout::new(SubwidgetPointer::new(
                    Box::new(move |s: &Self| &s.buttons[idx1]),
                    Box::new(move |s: &mut Self| &mut s.buttons[idx2]),
                ))
                .boxed()
            })
            .collect();

        let button_layout = button_layouts
            .into_iter()
            .fold(SplitLayout::new(SplitDirection::Horizontal), |acc, layout| {
                acc.with(SplitRule::Proportional(1.0), layout)
            })
            .boxed();

        let total_layout = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0), text_layout)
            .with(SplitRule::Fixed(1), button_layout)
            .boxed();

        let mut frame_layout = FrameLayout::new(total_layout, XY::new(1, 1));

        if self.border_op.is_some() {
            frame_layout = frame_layout.with_frame(todosubwidgetunwrap!(Self.border_op));
        }

        let frame_layout = frame_layout.boxed();

        frame_layout
    }

    fn get_default_focused(&self) -> SubwidgetPointer<GenericDialog> {
        if self.buttons.is_empty() {
            error!("no buttons in generic dialog!");
            subwidget!(Self.text_widget)
        } else {
            SubwidgetPointer::new(
                Box::new(|x: &Self| x.buttons.get(0).unwrap()),
                Box::new(|x: &mut Self| x.buttons.get_mut(0).unwrap()),
            )
        }
    }

    fn set_display_state(&mut self, display_state: DisplayState<GenericDialog>) {
        self.display_state = Some(display_state);
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<GenericDialog>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}
