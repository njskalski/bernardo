/*
this widget is supposed to offer:
- tree view on the right, along with scrolling,
- file list view on the most of display (with scrolling as well)
- filename edit box
- buttons save and cancel

I hope I will discover most of functional constraints while implementing it.
 */

use log::{debug, error, warn};

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::layout::empty_layout::EmptyLayout;
use crate::layout::frame_layout::FrameLayout;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::border::SINGLE_BORDER_STYLE;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::text::text_buffer::TextBuffer;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WidgetActionParam, WID};
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::generic_dialog::generic_dialog::GenericDialog;
use crate::widgets::list_widget::list_widget::ListWidget;
use crate::widgets::save_file_dialog::dialogs::override_dialog;
use crate::widgets::save_file_dialog::save_file_dialog_msg::SaveFileDialogMsg;
use crate::widgets::spath_tree_view_node::DirTreeNode;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;

// TODO now it displays both files and directories in tree view, it should only directories

pub struct SaveFileDialogWidget {
    id: WID,

    display_state: Option<DisplayState<Self>>,

    // TODO at this point I just want it to work, I should profile if it behaves fast.
    tree_widget: WithScroll<TreeViewWidget<SPath, DirTreeNode>>,
    list_widget: ListWidget<SPath>,
    edit_box: EditBoxWidget,

    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,

    on_cancel: Option<WidgetAction<Self>>,
    on_save: Option<WidgetActionParam<Self, SPath>>,

    root_path: SPath,

    hover_dialog: Option<GenericDialog>,
}

impl SaveFileDialogWidget {
    pub const TYPENAME: &'static str = "save_file_dialog";
    pub const OK_LABEL: &'static str = "OK";
    pub const CANCEL_LABEL: &'static str = "CANCEL";

    pub fn new(fsf: FsfRef, config: ConfigRef) -> Self {
        let root = fsf.root();

        let tree_widget = TreeViewWidget::<SPath, DirTreeNode>::new(DirTreeNode::new(root.clone()))
            .with_size_policy(SizePolicy::MATCH_LAYOUT)
            .with_on_flip_expand(Box::new(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::TreeExpanded(item.spath().clone())))
            }))
            .with_on_highlighted_changed(Box::new(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::TreeHighlighted(item.spath().clone())))
            }))
            .with_on_hit(Box::new(|widget| {
                let (_, item) = widget.get_highlighted();
                SaveFileDialogMsg::TreeHit(item.spath().clone()).someboxed()
            }));

        let scroll_tree_widget = WithScroll::new(ScrollDirection::Vertical, tree_widget);

        let list_widget: ListWidget<SPath> = ListWidget::new()
            .with_selection()
            .with_size_policy(SizePolicy::MATCH_LAYOUT)
            .with_on_hit(Box::new(|w| {
                w.get_highlighted()
                    .map(|item| Some(SaveFileDialogMsg::FileListHit(item.clone()).boxed()))
                    .flatten()
            }));
        let edit_box = EditBoxWidget::new(config)
            .with_size_policy(SizePolicy::MATCH_LAYOUT)
            .with_enabled(true)
            .with_on_hit(Box::new(|_| SaveFileDialogMsg::EditBoxHit.someboxed()));
        let ok_button = ButtonWidget::new(Box::new(Self::OK_LABEL)).with_on_hit(Box::new(|_| SaveFileDialogMsg::Save.someboxed()));
        let cancel_button =
            ButtonWidget::new(Box::new(Self::CANCEL_LABEL)).with_on_hit(Box::new(|_| SaveFileDialogMsg::Cancel.someboxed()));

        let path = fsf.root();

        SaveFileDialogWidget {
            id: get_new_widget_id(),
            display_state: None,
            tree_widget: scroll_tree_widget,
            list_widget,
            edit_box,
            ok_button,
            cancel_button,
            on_save: None,
            on_cancel: None,
            root_path: path,
            hover_dialog: None,
        }
    }

    /*
    Sets the path of expanded nodes in save file dialog, and potentially filename.
    If set to a dir, only tree will be expanded.
    If set to a file, both tree will be expanded, and editbox filled.
     */
    pub fn set_path(&mut self, path: SPath) -> bool {
        debug!("setting path to {:?}", path);

        let (dir, file): (Option<SPath>, Option<SPath>) = if path.is_file() {
            (path.parent(), Some(path))
        } else {
            (Some(path), None)
        };

        debug!("setting path to {:?}, {:?}", dir, file);

        let mut changed = false;
        if let Some(dir) = dir {
            debug_assert!(dir.is_dir());

            if let Some(parent) = dir.parent_ref() {
                changed = self.tree_widget.internal_mut().expand_path(parent);
            }

            if !self.tree_widget.internal_mut().set_selected(&dir) {
                error!("failed setting selected {:?}", dir);
            }

            self.show_files_on_right_panel(&dir);
        }

        if let Some(file) = file {
            self.edit_box.set_text(file.label().as_ref());
            self.edit_box.set_cursor_end();
            changed = true;
        }

        changed
    }

    // returns whether succeeded
    fn show_files_on_right_panel(&mut self, directory: &SPath) -> bool {
        if !directory.is_dir() {
            warn!("expected directory, got {:?}", directory);
            return false;
        }

        match directory.blocking_list() {
            Ok(items) => {
                let files: Vec<SPath> = items.into_iter().filter(|i| i.is_file()).collect();
                self.list_widget.set_provider(Box::new(files));
                true
            }
            Err(e) => {
                error!("failed to list {:?}: {:?}", directory, e);
                self.list_widget.set_provider(Box::new(()));
                false
            }
        }
    }

    pub fn with_path(mut self, path: SPath) -> Self {
        self.set_path(path);
        self
    }

    pub fn set_on_cancel(&mut self, on_cancel: WidgetAction<Self>) {
        self.on_cancel = Some(on_cancel);
    }

    pub fn with_on_cancel(self, on_cancel: WidgetAction<Self>) -> Self {
        Self {
            on_cancel: Some(on_cancel),
            ..self
        }
    }

    pub fn set_on_save(&mut self, on_save: WidgetActionParam<Self, SPath>) {
        self.on_save = Some(on_save);
    }

    pub fn with_on_save(self, on_save: WidgetActionParam<Self, SPath>) -> Self {
        Self {
            on_save: Some(on_save),
            ..self
        }
    }

    // TODO tests!
    pub fn get_path(&self) -> Option<SPath> {
        if self.edit_box.get_buffer().len_chars() == 0 {
            None
        } else {
            let file_name = self.edit_box.get_buffer().to_string();
            let path = self.tree_widget.internal().get_highlighted().1.spath().clone();
            let result = path.descendant_unchecked(file_name);

            if result.is_none() {
                error!("expected non-empty result");
            }

            result
        }
    }

    // Returns op message to parent, so it can be called from 'update'
    pub fn save_or_ask_for_override(&mut self) -> Option<Box<dyn AnyMsg>> {
        if self.hover_dialog.is_some() {
            error!("save_or_ask_for_override called from unexpected state");
        }

        let path = match self.get_path() {
            Some(p) => p,
            None => {
                error!("can't save of empty path. The OK button should have been blocked!");
                return None;
            }
        };

        if path.exists() {
            let filename = self.edit_box.get_buffer().to_string();

            self.hover_dialog = Some(override_dialog(filename));
            self.get_hover_pointer()
                .map(|ptr| {
                    self.set_focused(ptr);
                })
                .unwrap_or_else(|| {
                    error!("failed to set focus to hover dialog!");
                });
            return None;
        } else {
            self.save_positively()
        }
    }

    // Returns op message to parent, so it can be called from 'update'
    fn save_positively(&self) -> Option<Box<dyn AnyMsg>> {
        let path = match self.get_path() {
            Some(p) => p,
            None => {
                error!("self.get_path() is None in save_positively!");
                return None;
            }
        };

        self.on_save.as_ref().map(|on_save| on_save(self, path)).unwrap_or_else(|| {
            error!("attempted to save, but on_save not set");
            None
        })
    }

    fn get_hover_pointer(&mut self) -> Option<SubwidgetPointer<Self>> {
        if self.hover_dialog.is_some() {
            Some(SubwidgetPointer::new(
                Box::new(|s: &Self| {
                    if s.hover_dialog.is_some() {
                        s.hover_dialog.as_ref().unwrap()
                    } else {
                        error!("no hover found in save_file_dialog, this subwidget pointer should have been overriden by now.");
                        // debug_assert!(false, "no hover found, this subwidget pointer should have been overriden by now.");
                        let sw = s.get_default_focused().clone();
                        sw.get(s)
                    }
                }),
                Box::new(|s: &mut Self| {
                    if s.hover_dialog.is_some() {
                        s.hover_dialog.as_mut().unwrap()
                    } else {
                        error!("no hover found in save_file_dialog, this subwidget pointer should have been overriden by now.");
                        // debug_assert!(false, "no hover found, this subwidget pointer should have been overriden by now.");
                        let sw = s.get_default_focused().clone();
                        sw.get_mut(s)
                    }
                }),
            ))
        } else {
            None
        }
    }

    fn get_child_hover_rect(screenspace: Screenspace) -> Option<Rect> {
        let output_size = screenspace.output_size();
        if output_size >= XY::new(10, 8) {
            let max_size = output_size - XY::new(2, 2);
            let margins = max_size / 10;
            Some(Rect::new(margins, max_size - (margins * 2)))
        } else {
            None
        }
    }
}

impl Widget for SaveFileDialogWidget {
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
        XY::new(4, 4)
    }

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::MATCH_LAYOUT
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        // debug!("save_file_dialog.on_input {:?}", input_event);

        return match input_event {
            InputEvent::KeyInput(key) => match key.keycode {
                Keycode::Esc => SaveFileDialogMsg::Cancel.someboxed(),
                keycode if keycode.is_arrow() => {
                    if let (Some(msg), Some(ds)) = (key.as_focus_update(), &self.display_state) {
                        if ds.focus_group.can_update_focus(msg) {
                            SaveFileDialogMsg::FocusUpdateMsg(msg).someboxed()
                        } else {
                            None
                        }
                    } else {
                        error!("failed to cast arrow to focus update");
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        // debug!("save_file_dialog.update {:?}", msg);

        let our_msg = msg.as_msg::<SaveFileDialogMsg>();
        if our_msg.is_none() {
            warn!("expecetd SaveFileDialogMsg, got {:?}", msg);
            return None;
        }

        #[allow(unreachable_patterns)]
        return match our_msg.unwrap() {
            SaveFileDialogMsg::TreeExpanded(..) => None,
            SaveFileDialogMsg::TreeHighlighted(node) => {
                if !self.set_path(node.clone()) {
                    warn!("failed to set path {:?}", node);
                }

                None
            }
            SaveFileDialogMsg::TreeHit(_) => {
                self.set_focused(subwidget!(Self.list_widget));
                None
            }
            SaveFileDialogMsg::FileListHit(file) => {
                let text = file.file_name_str().unwrap();
                self.edit_box.set_text(text); // TODO
                self.edit_box.set_cursor_end();
                self.set_focused(subwidget!(Self.edit_box));
                None
            }
            SaveFileDialogMsg::EditBoxHit => {
                self.set_focused(subwidget!(Self.ok_button));
                None
            }
            SaveFileDialogMsg::Cancel => self
                .on_cancel
                .as_ref()
                .and_then(Box::new(|action: &WidgetAction<Self>| action(self))),
            SaveFileDialogMsg::Save => self.save_or_ask_for_override(),
            SaveFileDialogMsg::CancelOverride => {
                self.hover_dialog = None;
                None
            }
            SaveFileDialogMsg::FocusUpdateMsg(fu_msg) => {
                self.update_focus(*fu_msg);
                None
            }
            SaveFileDialogMsg::ConfirmOverride => {
                debug_assert!(self.hover_dialog.is_some());
                self.hover_dialog = None;
                self.set_focused(self.get_default_focused());
                self.save_positively()
            }
            unknown_msg => {
                warn!("SaveFileDialog.update : unknown message {:?}", unknown_msg);
                None
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
        #[cfg(any(test, feature = "fuzztest"))]
        {
            let size = crate::unpack_unit_e!(self.display_state.as_ref(), "render before layout",).total_size;

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
}

impl ComplexWidget for SaveFileDialogWidget {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let tree_layout = LeafLayout::new(subwidget!(Self.tree_widget));

        let left_column = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0), tree_layout.boxed())
            // .with(SplitRule::Fixed(1), EmptyLayout::new().with_size(XY::new(1, 1)).boxed())
            .boxed();

        let ok_box = LeafLayout::new(subwidget!(Self.ok_button)).boxed();
        let cancel_box = LeafLayout::new(subwidget!(Self.cancel_button)).boxed();

        let button_bar = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(10), cancel_box)
            .with(SplitRule::Fixed(10), ok_box)
            .boxed();

        let list = LeafLayout::new(subwidget!(Self.list_widget)).boxed();
        let edit = LeafLayout::new(subwidget!(Self.edit_box)).boxed();
        let right_column = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0), list)
            .with(SplitRule::Fixed(1), edit)
            .with(SplitRule::Fixed(1), button_bar)
            .boxed();

        let layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(2.0), left_column)
            .with(SplitRule::Proportional(3.0), right_column)
            .boxed();

        let frame = XY::new(1, 1);

        if self.hover_dialog.is_none() {
            FrameLayout::new(layout, frame).boxed()
        } else {
            //TODO(subwidgetpointermap)
            let dialog_layout = LeafLayout::new(SubwidgetPointer::new(
                Box::new(|x: &Self| x.hover_dialog.as_ref().unwrap()),
                Box::new(|x: &mut Self| x.hover_dialog.as_mut().unwrap()),
            ))
            .boxed();

            FrameLayout::new(
                HoverLayout::new(layout, dialog_layout, Box::new(Self::get_child_hover_rect), true).boxed(),
                frame,
            )
            .boxed()
        }
    }

    fn get_default_focused(&self) -> SubwidgetPointer<SaveFileDialogWidget> {
        subwidget!(Self.tree_widget)
    }

    fn set_display_state(&mut self, display_state: DisplayState<SaveFileDialogWidget>) {
        self.display_state = Some(display_state)
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<SaveFileDialogWidget>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}
