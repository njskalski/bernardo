use crate::config::theme::Theme;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::widget::any_msg::AnyMsg;
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::text_widget::TextWidget;

struct CodePreviewWidget {
    id: WID,
    label: TextWidget,
    editor: EditorWidget,

    display_state: Option<DisplayState<CodePreviewWidget>>,

    editor_lines: u16,
}

impl CodePreviewWidget {
    pub const TYPENAME: &'static str = "code_preview_widget";
    
    pub fn new(label: TextWidget, editor: EditorWidget) -> Self {
        CodePreviewWidget {
            id: get_new_widget_id(),
            label,
            editor,
            display_state: None,
            editor_lines: 3,
        }
    }
}

impl Widget for CodePreviewWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn size(&self) -> XY {
        let editor_size = self.editor.size();
        let label_size = self.label.size();

        XY::new(
            editor_size.x.max(label_size.x),
            self.editor_lines + 1,
        )
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        self.complex_layout(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.complex_render(theme, focused, output)
    }
}

impl ComplexWidget for CodePreviewWidget {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        SplitLayout::new(SplitDirection::Horizontal)
            .with(
                SplitRule::Fixed(1),
                LeafLayout::new(subwidget!(Self.label))
                    .with_focusable(false)
                    .boxed(),
            )
            .with(
                SplitRule::Fixed(3),
                LeafLayout::new(subwidget!(Self.editor)).boxed(),
            )
            .boxed()
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        subwidget!(Self.editor)
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