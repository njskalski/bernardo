use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::path::SPath;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::layout::empty_layout::EmptyLayout;
use crate::layout::frame_layout::FrameLayout;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::border::SINGLE_BORDER_STYLE;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::editor_widget::label::label::Label;
use crate::widgets::text_widget::TextWidget;

const FIND_IN_FILES_WIDGET_NAME: &'static str = "find_in_files_widget";

pub struct FindInFilesWidget {
    wid: WID,
    root: SPath,

    layout_result: Option<LayoutResult<Self>>,

    label: TextWidget,
    edit_box_widget: EditBoxWidget,
    search_button: ButtonWidget,
    cancel_button: ButtonWidget,
}

impl FindInFilesWidget {
    pub fn new(root: SPath) -> Self {
        FindInFilesWidget {
            wid: get_new_widget_id(),
            root,
            layout_result: None,
            label: TextWidget::new(Box::new("Search in files:")),
            edit_box_widget: EditBoxWidget::default(),
            search_button: ButtonWidget::new(Box::new("Search")),
            cancel_button: ButtonWidget::new(Box::new("Cancel")),
        }
    }
}

impl Widget for FindInFilesWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        FIND_IN_FILES_WIDGET_NAME
    }

    fn typename(&self) -> &'static str {
        FIND_IN_FILES_WIDGET_NAME
    }

    fn full_size(&self) -> XY {
        todo!()
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn prelayout(&mut self) {
        self.complex_prelayout()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            let size = crate::unpack_unit_e!(self.layout_result.as_ref(), "render before layout",).total_size;

            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: Rect::from_zero(size),
                focused,
            });
        }

        self.complex_render(theme, focused, output);
        SINGLE_BORDER_STYLE.draw_edges(theme.default_text(focused), output);
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.complex_get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.complex_get_focused_mut()
    }
}

impl ComplexWidget for FindInFilesWidget {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let frame = XY::new(1, 1);

        let text_line = LeafLayout::new(subwidget!(Self.label)).boxed();

        let edit = LeafLayout::new(subwidget!(Self.edit_box_widget)).boxed();

        let ok_box = LeafLayout::new(subwidget!(Self.search_button)).boxed();
        let cancel_box = LeafLayout::new(subwidget!(Self.cancel_button)).boxed();

        let button_bar = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(10), cancel_box)
            .with(SplitRule::Fixed(10), ok_box)
            .boxed();

        let combined_layout = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Fixed(1), text_line)
            .with(SplitRule::Fixed(1), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(1), edit)
            .with(SplitRule::Fixed(1), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(1), button_bar)
            .boxed();

        FrameLayout::new(combined_layout, frame).boxed()
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        todo!()
    }

    fn set_display_state(&mut self, display_state: DisplayState<Self>) {
        todo!()
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<Self>> {
        todo!()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        todo!()
    }
}
