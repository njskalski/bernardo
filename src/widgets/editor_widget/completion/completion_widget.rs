/*
I guess I should reuse FuzzySearch Widget, this is a placeholder now.
 */

use std::cmp::min;
use std::sync::{Arc, RwLock};

use log::{debug, error, warn};

use crate::{AnyMsg, InputEvent, Output, selfwidget, SizeConstraint, subwidget, Theme, Widget};
use crate::experiments::focus_group::FocusUpdate;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::xy::XY;
use crate::promise::promise::Promise;
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
    completions_promise: CompletionsPromise,
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
            completions_promise,
            display_state: None,
        }
    }

    fn completions(&self) -> Option<&Vec<Completion>> {
        self.completions_promise.read()
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

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        self.completions_promise.update();

        if self.completions_promise.read().is_some() {
            self.set_focused(subwidget!(Self.list_widget));
        }
        
        self.complex_layout(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<CompletionWidgetMsg>() {
            None => {
                warn!("expecetd CompletionWidgetMsg, got {:?}", msg);
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
        if self.completions().is_some() {
            Box::new(LeafLayout::new(subwidget!(Self.list_widget)))
        } else {
            Box::new(LeafLayout::new(selfwidget!(Self)))
        }
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        if self.completions().is_some() {
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
