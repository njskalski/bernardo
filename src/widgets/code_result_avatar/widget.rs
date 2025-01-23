use log::error;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::path::SPath;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::xy::XY;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::widget::any_msg::AnyMsg;
use crate::widget::combined_widget::CombinedWidget;
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::context_menu::widget::CONTEXT_MENU_WIDGET_NAME;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::text_widget::TextWidget;
use crate::{subwidget, unpack_unit_e};

pub struct CodeResultAvatarWidget {
    id: WID,
    editor_view: EditorView,
    label: TextWidget,

    layout_res: Option<LayoutResult<Self>>,
}

impl CodeResultAvatarWidget {
    pub const TYPENAME: &'static str = "code_result_avatar_widget";
    pub fn new(editor_view: EditorView) -> CodeResultAvatarWidget {
        let description = match editor_view.get_path() {
            None => "[no path]".to_string(),
            Some(path) => {
                format!("    ^ [{}] ^", path.to_string())
            }
        };

        CodeResultAvatarWidget {
            id: get_new_widget_id(),
            editor_view,
            label: TextWidget::new(Box::new(description)),
            layout_res: None,
        }
    }

    pub fn get_buffer_ref(&self) -> &BufferSharedRef {
        self.editor_view.get_buffer_ref()
    }

    pub fn get_editor_widget(&self) -> &EditorWidget {
        self.editor_view.get_internal_widget()
    }
}

impl Widget for CodeResultAvatarWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }

    fn prelayout(&mut self) {
        self.combined_prelayout()
    }

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::MATCH_LAYOUTS_WIDTH
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn full_size(&self) -> XY {
        XY::new(10, 5)
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.combined_layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            let size = unpack_unit_e!(self.get_layout_res().map(|lr| lr.total_size), "render before layout",);

            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: Self::TYPENAME.to_string(),
                rect: crate::primitives::rect::Rect::from_zero(size),
                focused,
            });
        }

        self.combined_render(theme, focused, output)
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        None
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        None
    }
}

impl CombinedWidget for CodeResultAvatarWidget {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        SplitLayout::new(SplitDirection::Vertical)
            .with(
                SplitRule::Proportional(1.0f32),
                LeafLayout::new(subwidget!(Self.editor_view)).boxed(),
            )
            .with(SplitRule::Fixed(1), LeafLayout::new(subwidget!(Self.label)).boxed())
            .boxed()
    }

    fn save_layout_res(&mut self, result: crate::layout::layout::LayoutResult<Self>) {
        self.layout_res = Some(result);
    }

    fn get_layout_res(&self) -> Option<&crate::layout::layout::LayoutResult<Self>> {
        self.layout_res.as_ref()
    }

    fn get_subwidgets_for_input(&self) -> impl Iterator<Item = SubwidgetPointer<Self>> {
        [subwidget!(Self.editor_view), subwidget!(Self.label)].into_iter()
    }
}
