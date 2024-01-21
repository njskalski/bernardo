use log::{error, warn};

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::common_edit_msgs::{key_to_edit_msg, CommonEditMsg};
use crate::primitives::common_query::CommonQuery;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::text::buffer_state::BufferState;
use crate::widget::any_msg::AnyMsg;
use crate::widget::any_msg::AsAny;
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::editor_widget::context_bar::context_bar_item::ContextBarItem;
use crate::widgets::editor_widget::context_bar::msg::ContextBarWidgetMsg;
use crate::widgets::editor_widget::msg::EditorWidgetMsg;
use crate::widgets::list_widget::list_widget::ListWidget;

pub struct ContextBarWidget {
    id: WID,
    list: ListWidget<ContextBarItem>,

    display_state: Option<DisplayState<Self>>,

    query: BufferState,
}

impl ContextBarWidget {
    pub const TYPENAME: &'static str = "context_bar";

    pub fn new(items: Vec<ContextBarItem>) -> Self {
        ContextBarWidget {
            id: get_new_widget_id(),
            list: ListWidget::new()
                .with_selection()
                .with_provider(Box::new(items))
                .with_show_column_names(false)
                .with_size_policy(SizePolicy::MATCH_LAYOUTS_WIDTH)
                .with_on_hit(|_| ContextBarWidgetMsg::Hit.someboxed()),
            display_state: None,
            query: BufferState::simplified_single_line(),
        }
    }

    fn on_query_change(&mut self) {
        let query_str = self.query.to_string();
        if query_str.is_empty() {
            self.list.set_query(None);
        } else {
            let query = CommonQuery::Fuzzy(self.query.to_string());
            self.list.set_query(Some(query));
        }
    }
}

impl Widget for ContextBarWidget {
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

    fn prelayout(&mut self) {
        self.complex_prelayout();
    }

    fn full_size(&self) -> XY {
        XY::new(1, 12) // TODO completely arbitrary
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) if key.keycode == Keycode::Esc => ContextBarWidgetMsg::Close.someboxed(),
            InputEvent::KeyInput(key) if key_to_edit_msg(key).is_some() => {
                let msg = key_to_edit_msg(key).unwrap();

                let ignore: bool = match msg {
                    CommonEditMsg::Char(_) => false,
                    CommonEditMsg::Block(_) => true,
                    CommonEditMsg::CursorUp { .. } => true,
                    CommonEditMsg::CursorDown { .. } => true,
                    CommonEditMsg::CursorLeft { .. } => false,
                    CommonEditMsg::CursorRight { .. } => false,
                    CommonEditMsg::Backspace => false,
                    CommonEditMsg::LineBegin { .. } => true,
                    CommonEditMsg::LineEnd { .. } => true,
                    CommonEditMsg::WordBegin { .. } => true,
                    CommonEditMsg::WordEnd { .. } => true,
                    CommonEditMsg::PageUp { .. } => true,
                    CommonEditMsg::PageDown { .. } => true,
                    CommonEditMsg::Delete => true,
                    CommonEditMsg::Copy => true,
                    CommonEditMsg::Paste => true,
                    CommonEditMsg::Undo => true,
                    CommonEditMsg::Redo => true,
                    CommonEditMsg::DeleteBlock { .. } => true,
                    CommonEditMsg::InsertBlock { .. } => true,
                    CommonEditMsg::SubstituteBlock { .. } => true,
                    CommonEditMsg::Tab => true,
                    CommonEditMsg::ShiftTab => true,
                };

                if !ignore {
                    ContextBarWidgetMsg::Edit(msg).someboxed()
                } else {
                    None
                }
            }
            _ => {
                error!("unhandled msg {:?}", input_event);
                None
            }
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        #[allow(unreachable_patterns)]
        return match msg.as_msg::<ContextBarWidgetMsg>() {
            None => {
                warn!("expected ContextBarWidgetMsg, got {:?}", msg);
                None
            }
            Some(msg) => match msg {
                ContextBarWidgetMsg::Close => EditorWidgetMsg::HoverClose.someboxed(),
                ContextBarWidgetMsg::Edit(cem) => {
                    if self.query.apply_cem(cem.clone(), self.id, 1, None) {
                        self.on_query_change();
                    }
                    None
                }
                ContextBarWidgetMsg::Hit => self.list.get_highlighted_item().map(|item| item.msg()),
                _ => {
                    warn!("ignoring message {:?}", msg);
                    None
                }
            },
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            if let Some(ds) = self.get_display_state_op() {
                output.emit_metadata(crate::io::output::Metadata {
                    id: self.id,
                    typename: self.typename().to_string(),
                    rect: crate::primitives::rect::Rect::new(XY::ZERO, ds.total_size),
                    focused,
                });
            }
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

impl ComplexWidget for ContextBarWidget {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        Box::new(LeafLayout::new(subwidget!(Self.list)))
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        subwidget!(Self.list)
    }

    fn set_display_state(&mut self, display_state: DisplayState<Self>) {
        self.display_state = Some(display_state)
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<Self>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}
