use log::{debug, warn};

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::path::SPath;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::layout::empty_layout::EmptyLayout;
use crate::layout::frame_layout::FrameLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::border::SINGLE_BORDER_STYLE;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WID};
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::find_in_files_widget::msg::Msg;
use crate::widgets::text_widget::TextWidget;

const FIND_IN_FILES_WIDGET_NAME: &'static str = "find_in_files_widget";

pub struct FindInFilesWidget {
    wid: WID,
    root: SPath,

    label: TextWidget,

    query_box_label: TextWidget,
    query_box: EditBoxWidget,

    filter_box_label: TextWidget,
    filter_box: EditBoxWidget,

    search_button: ButtonWidget,
    cancel_button: ButtonWidget,

    display_state: Option<DisplayState<Self>>,

    on_hit: Option<WidgetAction<Self>>,
    on_cancel: Option<WidgetAction<Self>>,
}

impl FindInFilesWidget {
    const DEFAULT_SIZE: XY = XY::new(50, 10);

    pub fn new(root: SPath) -> Self {
        FindInFilesWidget {
            wid: get_new_widget_id(),
            root,
            label: TextWidget::new(Box::new("Search in files:")),
            query_box_label: TextWidget::new(Box::new("What:")),
            query_box: EditBoxWidget::default()
                .with_size_policy(SizePolicy::MATCH_LAYOUTS_WIDTH)
                .with_on_hit(Box::new(|_| Msg::Hit.someboxed())),
            filter_box_label: TextWidget::new(Box::new("Where:")),
            filter_box: EditBoxWidget::default().with_size_policy(SizePolicy::MATCH_LAYOUTS_WIDTH),
            search_button: ButtonWidget::new(Box::new("Search")).with_on_hit(Box::new(|_| Msg::Hit.someboxed())),
            cancel_button: ButtonWidget::new(Box::new("Cancel")).with_on_hit(Box::new(|_| Msg::Cancel.someboxed())),
            display_state: None,
            on_hit: None,
            on_cancel: None,
        }
    }

    pub fn with_on_hit(self, on_hit: Option<WidgetAction<Self>>) -> Self {
        Self { on_hit, ..self }
    }

    pub fn set_on_hit(&mut self, on_hit: Option<WidgetAction<Self>>) {
        self.on_hit = on_hit;
    }

    pub fn with_on_cancel(self, on_cancel: Option<WidgetAction<Self>>) -> Self {
        Self { on_cancel, ..self }
    }

    pub fn cancel(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_cancel.as_ref().map(|action| action(self)).flatten()
    }

    pub fn hit(&self) -> Option<Box<dyn AnyMsg>> {
        self.on_hit.as_ref().map(|action| action(self)).flatten()
    }

    pub fn set_on_cancel(&mut self, on_cancel: Option<WidgetAction<Self>>) {
        self.on_cancel = on_cancel;
    }

    pub fn get_query(&self) -> String {
        self.query_box.get_text()
    }

    pub fn get_filter(&self) -> Option<String> {
        let filter = self.filter_box.get_text();
        if filter.len() > 0 {
            Some(filter)
        } else {
            None
        }
    }

    pub fn is_focused_on_button(&self) -> bool {
        if let Some(focused) = self.display_state.as_ref().map(|ds| ds.focused.clone()) {
            let focused_widget_id = focused.get(self).id();
            if focused_widget_id == self.search_button.id() || focused_widget_id == self.cancel_button.id() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn root(&self) -> &SPath {
        &self.root
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
        Self::DEFAULT_SIZE
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn prelayout(&mut self) {
        self.complex_prelayout()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        match input_event {
            InputEvent::KeyInput(key) if key == Keycode::Esc.to_key() => Msg::Cancel.someboxed(),
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("update {:?}, receives {:?}", self as &dyn Widget, &msg);
        return match msg.as_msg::<Msg>() {
            None => {
                warn!("expecetd FindInFiles::Msg, got {:?}", msg);
                None
            }
            Some(msg) => match msg {
                Msg::Hit => self.on_hit.as_ref().map(|f| f(self)).flatten(),
                Msg::Cancel => self.on_cancel.as_ref().map(|f| f(self)).flatten(),
            },
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            let size = crate::unpack_unit_e!(self.display_state.as_ref().map(|item| item.total_size), "render before layout",);

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
    fn pre_act_on(&mut self, input_event: &InputEvent) {
        // This is implementation of a thing:
        //  if user started typing while pointing at buttons, it will jump to query immediately
        if input_event.as_key().map(|key| key.keycode.is_symbol()).unwrap_or(false) {
            if self.is_focused_on_button() {
                self.set_focused(subwidget!(Self.query_box));
            }
        }
    }
}

impl ComplexWidget for FindInFilesWidget {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let frame = XY::new(1, 1);

        let text_line = LeafLayout::new(subwidget!(Self.label)).boxed();

        let query = LeafLayout::new(subwidget!(Self.query_box)).boxed();
        let query_label = LeafLayout::new(subwidget!(Self.query_box_label)).boxed();

        let query_line = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Fixed(7), query_label)
            .with(SplitRule::Proportional(1.0), query);

        let filter = LeafLayout::new(subwidget!(Self.filter_box)).boxed();
        let filter_label = LeafLayout::new(subwidget!(Self.filter_box_label)).boxed();

        let filter_line = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Fixed(7), filter_label)
            .with(SplitRule::Proportional(1.0), filter);

        let ok_box = LeafLayout::new(subwidget!(Self.search_button)).boxed();
        let cancel_box = LeafLayout::new(subwidget!(Self.cancel_button)).boxed();

        let button_bar = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(10), ok_box)
            .with(SplitRule::Fixed(10), cancel_box)
            .boxed();

        let combined_layout = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Fixed(1), text_line)
            .with(SplitRule::Fixed(1), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(1), query_line.boxed())
            // .with(SplitRule::Fixed(1), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(1), filter_line.boxed())
            .with(SplitRule::Fixed(1), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(1), button_bar)
            .boxed();

        FrameLayout::new(combined_layout, frame).boxed()
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        subwidget!(Self.search_button)
    }

    fn set_display_state(&mut self, display_state: DisplayState<Self>) {
        self.display_state = Some(display_state);
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<Self>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}
