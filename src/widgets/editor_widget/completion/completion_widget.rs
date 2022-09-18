/*
I guess I should reuse FuzzySearch Widget, this is a placeholder now.
 */

use std::cmp::min;
use std::mem;
use std::sync::{Arc, RwLock};

use log::{debug, error, warn};

use crate::{AnyMsg, InputEvent, Output, selfwidget, SizeConstraint, subwidget, Theme, Widget};
use crate::experiments::focus_group::FocusUpdate;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::xy::XY;
use crate::promise::promise::{Promise, PromiseState};
use crate::w7e::navcomp_provider::{Completion, CompletionsPromise};
use crate::widget::action_trigger::ActionTrigger;
use crate::widget::any_msg::AsAny;
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_widget::completion::msg::CompletionWidgetMsg;
use crate::widgets::editor_widget::msg::EditorWidgetMsg;
use crate::widgets::fuzzy_search::fuzzy_search::FuzzySearchWidget;
use crate::widgets::list_widget::list_widget::ListWidget;

pub struct CompletionWidget {
    wid: WID,
    /*
     I store the promise until it's resolved, and then either keep broken promise OR move it's
     contents to list_widget. If promise got broken, I expect EditorWidget to throw away entire
     CompletionWidget in it's update_and_layout() and log error.
     */
    completions_promise: Option<CompletionsPromise>,
    fuzzy: bool,
    list_widget: ListWidget<Completion>,
    display_state: Option<DisplayState<Self>>,
}

impl CompletionWidget {
    pub fn new(completions_promise: CompletionsPromise) -> Self {
        CompletionWidget {
            wid: get_new_widget_id(),
            fuzzy: true,
            list_widget: ListWidget::new(),
            completions_promise: Some(completions_promise),
            display_state: None,
        }
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
                            let mut promise = None;
                            mem::swap(&mut self.completions_promise, &mut promise);
                            let value = promise.take().unwrap();
                            self.list_widget.set_provider(Box::new(value));
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
        "CompletionWidget"
    }

    fn min_size(&self) -> XY {
        // TODO completely arbitrary
        (10, 1).into()
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        self.completions_promise.as_mut().map(|cp| {
            if cp.update().has_changed {}
        });

        self.complex_layout(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
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
                _ => {
                    warn!("ignoring message {:?}", msg);
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.complex_render(theme, focused, output)
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.complex_get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.complex_get_focused_mut()
    }
}

impl ComplexWidget for CompletionWidget {
    fn get_layout(&self, max_size: XY) -> Box<dyn Layout<Self>> {
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
        for x in 0..output.size_constraint().visible_hint().size.x {
            for y in 0..output.size_constraint().visible_hint().size.y {
                output.print_at(XY::new(x, y), theme.ui.focused_highlighted, "!");
            }
        }
    }
}
