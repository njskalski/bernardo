use log::{debug, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::cursor::cursor_set::CursorSet;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::common_edit_msgs::{key_to_edit_msg, CommonEditMsg};
use crate::primitives::helpers;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WID};
use crate::{unpack_or_e, unpack_unit_e};

//TODO filter out the newlines on paste
//TODO add layout tests (min size, max size etc)

pub struct EditBoxWidget {
    id: WID,
    enabled: bool,
    // hit is basically pressing enter.
    on_hit: Option<WidgetAction<EditBoxWidget>>,
    on_change: Option<WidgetAction<EditBoxWidget>>,
    // miss is trying to make illegal move. Like backspace on empty, left on leftmost etc.
    on_miss: Option<WidgetAction<EditBoxWidget>>,
    buffer: BufferState,

    min_width_op: Option<u16>,
    max_width_op: Option<u16>,

    clipboard_op: Option<ClipboardRef>,

    last_size_x: Option<u16>,

    size_policy: SizePolicy,
    config: ConfigRef,
}

impl EditBoxWidget {
    const MIN_WIDTH: u16 = 2;

    pub const TYPENAME: &'static str = "edit_box";

    pub fn new(config: ConfigRef) -> Self {
        let widget_id = get_new_widget_id();
        let mut buffer = BufferState::simplified_single_line();
        buffer.initialize_for_widget(widget_id, None);

        let mut res = EditBoxWidget {
            id: widget_id,
            enabled: true,
            buffer,
            on_hit: None,
            on_change: None,
            on_miss: None,
            max_width_op: None,
            clipboard_op: None,
            last_size_x: None,
            min_width_op: None,
            size_policy: SizePolicy::SELF_DETERMINED,
            config,
        };

        res
    }

    pub fn with_size_policy(self, size_policy: SizePolicy) -> Self {
        Self { size_policy, ..self }
    }

    pub fn with_clipboard(self, clipboard: ClipboardRef) -> Self {
        Self {
            clipboard_op: Some(clipboard),
            ..self
        }
    }

    pub fn with_max_width(self, max_width: u16) -> Self {
        EditBoxWidget {
            max_width_op: Some(max_width),
            ..self
        }
    }

    pub fn with_min_width(self, min_width: u16) -> Self {
        EditBoxWidget {
            min_width_op: Some(min_width),
            ..self
        }
    }

    pub fn with_on_hit(self, on_hit: WidgetAction<EditBoxWidget>) -> Self {
        EditBoxWidget {
            on_hit: Some(on_hit),
            ..self
        }
    }

    pub fn with_on_change(self, on_change: WidgetAction<EditBoxWidget>) -> Self {
        EditBoxWidget {
            on_change: Some(on_change),
            ..self
        }
    }

    pub fn with_on_miss(self, on_miss: WidgetAction<EditBoxWidget>) -> Self {
        EditBoxWidget {
            on_miss: Some(on_miss),
            ..self
        }
    }

    pub fn with_enabled(self, enabled: bool) -> Self {
        EditBoxWidget { enabled, ..self }
    }

    pub fn with_text<'a, T: AsRef<str>>(self, text: T) -> Self {
        let mut res = EditBoxWidget {
            buffer: BufferState::simplified_single_line().with_text(text),
            ..self
        };
        res.buffer.initialize_for_widget(self.id, None);
        res
    }

    pub fn get_buffer(&self) -> &BufferState {
        &self.buffer
    }

    pub fn get_text(&self) -> String {
        self.buffer.text().to_string()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.len_bytes() == 0 //TODO
    }

    pub fn set_text<'a, T: AsRef<str>>(&mut self, text: T) {
        self.buffer = BufferState::simplified_single_line().with_text(text);
        self.buffer.initialize_for_widget(self.id, None);
    }

    pub fn set_cursor_end(&mut self) {
        let mut cursor_set = CursorSet::single();
        cursor_set.move_end(&self.buffer, false);
        self.buffer.text_mut().set_cursor_set(self.id, cursor_set);
    }

    pub fn clear(&mut self) {
        self.buffer = BufferState::simplified_single_line();
        self.buffer.initialize_for_widget(self.id, None);
    }

    fn event_changed(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_change.is_some() {
            self.on_change.as_ref().unwrap()(self)
        } else {
            None
        }
    }

    fn event_miss(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_miss.is_some() {
            self.on_miss.as_ref().unwrap()(self)
        } else {
            None
        }
    }

    fn event_hit(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_hit.is_some() {
            self.on_hit.as_ref().unwrap()(self)
        } else {
            None
        }
    }
}

impl Widget for EditBoxWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn full_size(&self) -> XY {
        XY::new(self.min_width_op.unwrap_or(Self::MIN_WIDTH), 1)
    }

    fn size_policy(&self) -> SizePolicy {
        self.size_policy
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.last_size_x = Some(screenspace.output_size().x);
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug_assert!(self.enabled, "EditBoxWidgetMsg: received input to disabled component!");

        let cursor_set_copy = unpack_or_e!(self.buffer.text().get_cursor_set(self.id), None, "failed to get cursor_set").clone();

        return match input_event {
            KeyInput(key_event) => {
                if key_event.keycode == Keycode::Enter {
                    Some(Box::new(EditBoxWidgetMsg::Hit))
                } else {
                    match key_to_edit_msg(key_event, &self.config.keyboard_config.edit_msgs) {
                        Some(cem) => match cem {
                            // the 4 cases below are designed to NOT consume the event in case it cannot be used.
                            CommonEditMsg::CursorUp { selecting: _ } | CommonEditMsg::CursorDown { selecting: _ } => None,
                            CommonEditMsg::CursorLeft { selecting: _ }
                                if cursor_set_copy.as_single().map(|c| c.a == 0).unwrap_or(false) =>
                            {
                                None
                            }
                            CommonEditMsg::CursorRight { selecting: _ }
                                if cursor_set_copy.as_single().map(|c| c.a > self.buffer.len_chars()).unwrap_or(false) =>
                            {
                                None
                            }
                            _ => Some(Box::new(EditBoxWidgetMsg::CommonEditMsg(cem))),
                        },
                        None => None,
                    }
                }
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<EditBoxWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd EditBoxWidgetMsg, got {:?}", msg);
            return None;
        }
        debug!("EditBox got {:?}", msg);

        return match our_msg.unwrap() {
            EditBoxWidgetMsg::Hit => self.event_hit(),
            EditBoxWidgetMsg::CommonEditMsg(cem) => {
                if self
                    .buffer
                    .apply_common_edit_message(cem.clone(), self.id, 1, self.clipboard_op.as_ref(), false)
                    .modified_buffer
                {
                    self.event_changed()
                } else {
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let size = XY::new(unpack_unit_e!(self.last_size_x, "render before layout",), 1);
        #[cfg(any(test, feature = "fuzztest"))]
        {
            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: crate::primitives::rect::Rect::from_zero(size),
                focused,
            });
        }

        let primary_style = theme.highlighted(focused);
        helpers::fill_output(primary_style.background, output);

        let cursor_set_copy = unpack_unit_e!(self.buffer.text().get_cursor_set(self.id), "failed to get cursor_set",).clone();

        let mut x: usize = 0;
        for (char_idx, g) in self.buffer.to_string().graphemes(true).enumerate() {
            if x + g.width() > size.x as usize {
                // not drawing beyond x
                break;
            }

            let style = match theme.cursor_background(cursor_set_copy.get_cursor_status_for_char(char_idx)) {
                Some(bg) => primary_style.with_background(if focused { bg } else { bg.half() }),
                None => primary_style,
            };
            output.print_at(
                XY::new(x as u16, 0), //TODO
                style,
                g,
            );
            x += g.width();
        }
        // one character after, but only if it fits.
        if x < size.x as usize {
            let style = match theme.cursor_background(cursor_set_copy.get_cursor_status_for_char(self.buffer.len_chars())) {
                Some(bg) => primary_style.with_background(if focused { bg } else { bg.half() }),
                None => primary_style,
            };
            output.print_at(XY::new(x as u16, 0), style, " ");
        }

        // if cursor is after the text, we need to add an offset, so the background does not
        // overwrite cursor style.
        let cursor_offset: u16 = cursor_set_copy.max_cursor_pos() as u16 + 1; //TODO
        let text_width = self.buffer.to_string().width() as u16; //TODO
        let end_of_text = cursor_offset.max(text_width);

        // background after the text
        if size.x > end_of_text {
            let background_length = size.x - end_of_text;
            for i in 0..background_length {
                let pos = XY::new(end_of_text + i as u16, 0);

                output.print_at(pos, primary_style, " ");
            }
        }
    }

    fn kite(&self) -> XY {
        XY::ZERO
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditBoxWidgetMsg {
    Hit,
    CommonEditMsg(CommonEditMsg),
}

impl AnyMsg for EditBoxWidgetMsg {}
