/*
I guess I should reuse FuzzySearch Widget, this is a placeholder now.
 */

use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::{selfwidget, subwidget};
use crate::config::theme::Theme;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::{Metadata, Output};
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::common_query::CommonQuery;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::promise::promise::PromiseState;
use crate::w7e::navcomp_provider::{Completion, CompletionsPromise};
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::FillPolicy;
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::editor_widget::completion::msg::CompletionWidgetMsg;
use crate::widgets::editor_widget::msg::EditorWidgetMsg;
use crate::widgets::list_widget::list_widget::ListWidget;

pub struct CompletionWidget {
    wid: WID,
    /*
     I store the promise until it's resolved, and then either keep broken promise OR move it's
     contents to list_widget. If promise got broken, I expect EditorWidget to throw away entire
     CompletionWidget in it's update_and_layout() and log error.
     */
    completions_promise: Option<CompletionsPromise>,

    list_widget: ListWidget<Completion>,
    display_state: Option<DisplayState<Self>>,

    fuzzy: bool,
}

impl CompletionWidget {
    pub const LOADING: &'static str = "loading...";
    pub const TYPENAME: &'static str = "completion_widget";

    pub fn new(completions_promise: CompletionsPromise) -> Self {
        CompletionWidget {
            wid: get_new_widget_id(),
            list_widget: ListWidget::new()
                .with_selection()
                .with_show_column_names(false)
                .with_fill_policy(FillPolicy::FILL_WIDTH)
                .with_on_hit(|w| {
                    w.get_highlighted().map(|c| {
                        CompletionWidgetMsg::Selected(c.action.clone()).boxed()
                    })
                })
            ,
            completions_promise: Some(completions_promise),
            display_state: None,
            fuzzy: false,
        }
    }

    pub fn with_fuzzy(self, fuzzy: bool) -> Self {
        Self {
            fuzzy,
            ..self
        }
    }

    pub fn set_fuzzy(&mut self, fuzzy: bool) {
        self.fuzzy = fuzzy;
    }

    pub fn get_fuzzy(&self) -> bool {
        self.fuzzy
    }

    pub fn set_query_substring(&mut self, query: Option<String>) {
        self.list_widget.set_query(query.map(|q|
            if !self.fuzzy {
                CommonQuery::String(q)
            } else {
                CommonQuery::Fuzzy(q)
            }
        ));
        debug!("updated query: {:?}", self.list_widget.get_query());
    }

    fn has_completions(&self) -> bool {
        self.completions_promise.is_none()
    }

    /*
    Returns whether it should be drawn (true) or can be discarded (false)
     */
    pub fn should_draw(&mut self) -> bool {
        match self.completions_promise.as_mut() {
            None => {
                /*
                 This indicates, that completions are executed correctly and have been moved away
                 from promise to ListWidget.
                 */
                true
            }
            Some(promise) => {
                let update = promise.update();
                if update.has_changed {
                    match update.state {
                        PromiseState::Unresolved => {
                            error!("changed to unresolved? makes no sense. Discarding entire widget.");
                            false
                        }
                        PromiseState::Ready => {
                            // let mut promise: Option<Box<dyn Promise<Vec<Completion>>>> = None;
                            // mem::swap(&mut self.completions_promise, &mut promise);
                            // let provider: Vec<Completion> = promise.unwrap().take().unwrap();
                            let provider = self.completions_promise.take().unwrap();
                            self.list_widget.set_provider(Box::new(provider));

                            self.set_focused(subwidget!(Self.list_widget));
                            true
                        }
                        PromiseState::Broken => {
                            error!("discarding completion widget due to broken promise : {:?}", promise);
                            false
                        }
                    }
                } else {
                    // still updating
                    true
                }
            }
        }
    }
}

impl Widget for CompletionWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn min_size(&self) -> XY {
        XY::new(Self::LOADING.width() as u16, 1)
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        self.completions_promise.as_mut().map(|cp| {
            if cp.update().has_changed {}
        });

        self.complex_layout(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) if key.keycode == Keycode::Esc => {
                CompletionWidgetMsg::Close.someboxed()
            }
            _ => None
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<CompletionWidgetMsg>() {
            None => {
                warn!("expected CompletionWidgetMsg, got {:?}", msg);
                None
            }
            Some(msg) => match msg {
                CompletionWidgetMsg::Close => {
                    EditorWidgetMsg::CompletionWidgetClose.someboxed()
                }
                CompletionWidgetMsg::Selected(action) => {
                    EditorWidgetMsg::CompletionWidgetSelected(action.clone()).someboxed()
                }
                _ => {
                    warn!("ignoring message {:?}", msg);
                    None
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
        #[cfg(test)]
        {
            if let Some(ds) = self.get_display_state_op() {
                let size = ds.todo_size();
                output.emit_metadata(Metadata {
                    id: self.wid,
                    typename: self.typename(),
                    rect: Rect::new(XY::ZERO, size),
                });
            }
        }

        self.complex_render(theme, focused, output)
    }
}

impl ComplexWidget for CompletionWidget {
    fn get_layout(&self, _max_size: XY) -> Box<dyn Layout<Self>> {
        if self.has_completions() {
            Box::new(LeafLayout::new(subwidget!(Self.list_widget)))
        } else {
            Box::new(LeafLayout::new(selfwidget!(Self)))
        }
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        if self.has_completions() {
            subwidget!(Self.list_widget)
        } else {
            selfwidget!(Self)
        }
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

    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        output.print_at(XY::ZERO, theme.highlighted(focused), Self::LOADING);
    }
}
