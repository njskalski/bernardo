use log::warn;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::layout::frame_layout::FrameLayout;
use crate::layout::layout::{Layout, LayoutResult};
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::border::SINGLE_BORDER_STYLE;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::combined_widget::CombinedWidget;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::context_menu::widget::ContextMenuWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::fuzzy_search::fsf_provider::FsfProvider;
use crate::widgets::fuzzy_search::fuzzy_search::{DrawComment, FuzzySearchWidget};
use crate::widgets::fuzzy_search::item_provider::ItemsProvider;
use crate::widgets::fuzzy_search::msg::FuzzySearchMsg;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::text_widget::TextWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;

pub struct FuzzyFileSearchWidget {
    wid: WID,
    size: XY,

    title: TextWidget,
    search_widget: WithScroll<FuzzySearchWidget>,

    layout_res: Option<LayoutResult<Self>>,
}

impl FuzzyFileSearchWidget {
    pub const TYPENAME: &'static str = "fuzzy_file_search";

    pub fn new(providers: &Providers, size: XY, provider: Box<dyn ItemsProvider>) -> Self {
        let search_widget = FuzzySearchWidget::new(|widget| MainViewMsg::CloseHover.someboxed(), Some(providers.clipboard().clone()))
            .with_size_policy(SizePolicy::MATCH_LAYOUT)
            .with_provider(provider)
            .with_draw_comment_setting(DrawComment::All);

        FuzzyFileSearchWidget {
            wid: get_new_widget_id(),
            title: TextWidget::new(Box::new("Fuzzy file search")),
            size,
            search_widget: WithScroll::new(ScrollDirection::Vertical, search_widget),
            layout_res: None,
        }
    }
}

impl Widget for FuzzyFileSearchWidget {
    fn id(&self) -> WID {
        self.wid
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
        self.size
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.combined_layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        self.search_widget.on_input(input_event)
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        Some(msg)
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            let size = crate::unpack_unit_e!(self.layout_res.as_ref(), "render before layout",).total_size;

            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: Self::TYPENAME.to_string(),
                rect: Rect::from_zero(size),
                focused,
            });
        }

        self.combined_render(theme, focused, output);
        // TODO merge border with title?
        SINGLE_BORDER_STYLE.draw_edges(theme.default_text(focused), output);
        self.title.render(theme, focused, output);
    }

    fn act_on(&mut self, input_event: InputEvent) -> (bool, Option<Box<dyn AnyMsg>>) {
        self.combined_act_on(input_event)
    }
}

impl CombinedWidget for FuzzyFileSearchWidget {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        FrameLayout::new(LeafLayout::new(subwidget!(Self.search_widget)).boxed(), XY::new(1, 1)).boxed()
    }

    fn save_layout_res(&mut self, result: LayoutResult<Self>) {
        self.layout_res = Some(result)
    }

    fn get_layout_res(&self) -> Option<&LayoutResult<Self>> {
        self.layout_res.as_ref()
    }

    fn get_subwidgets_for_input(&self) -> impl Iterator<Item = SubwidgetPointer<Self>> {
        [subwidget!(Self.search_widget)].into_iter()
    }
}
