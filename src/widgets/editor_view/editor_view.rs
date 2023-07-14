use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::{subwidget, unpack_or, unpack_or_e};
use crate::config::theme::Theme;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::path::SPath;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::output::{Metadata, Output};
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::primitives::has_invariant::HasInvariant;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::search_pattern::SearchPattern;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::text::buffer_state::{BufferState, SetFilePathResult};
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::w7e::handler::NavCompRef;
use crate::w7e::navcomp_group::NavCompGroupRef;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::editor_view::msg::EditorViewMsg;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogWidget;
use crate::widgets::text_widget::TextWidget;
use crate::widgets::with_scroll::WithScroll;

const PATTERN: &'static str = "pattern: ";
const REPLACE: &'static str = "replace: ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EditorViewState {
    Simple,
    Find,
    FindReplace,
}

// TODO join paths of saving file and set navcomp then in one place

pub struct EditorView {
    wid: WID,
    providers: Providers,

    display_state: Option<DisplayState<EditorView>>,

    editor: WithScroll<EditorWidget>,
    find_box: EditBoxWidget,
    find_label: TextWidget,
    replace_box: EditBoxWidget,
    replace_label: TextWidget,

    state: EditorViewState,
    hover_dialog: Option<SaveFileDialogWidget>,

    /*
    This represents "where the save as dialog should start", but only in case the file_front on buffer_state is None.
    If none, we'll use the fsf root.
    See get_save_file_dialog_path for details.
     */
    start_path: Option<SPath>,
}

impl EditorView {
    pub const TYPENAME: &'static str = "editor_view";

    pub fn new(
        providers: Providers,// TODO(#17) now navcomp is language specific, and editor can be "recycled" from say yaml to rs, requiring change of navcomp.
        buffer: BufferSharedRef,
    ) -> Self {
        let editor = EditorWidget::new(providers.clone(),
                                       buffer,
        );

        let find_label = TextWidget::new(Box::new(PATTERN));
        let replace_label = TextWidget::new(Box::new(REPLACE));

        let find_box = EditBoxWidget::new()
            .with_on_hit(|_| {
                EditorViewMsg::FindHit.someboxed()
            })
            .with_fill_x()
            .with_clipboard(providers.clipboard().clone());
        let replace_box = EditBoxWidget::new()
            .with_on_hit(|_| {
                EditorViewMsg::ReplaceHit.someboxed()
            })
            .with_fill_x()
            .with_clipboard(providers.clipboard().clone());

        EditorView {
            wid: get_new_widget_id(),
            providers,
            display_state: None,
            editor: WithScroll::new(ScrollDirection::Both, editor).with_line_no(),
            find_box,
            find_label,
            replace_box,
            replace_label,
            state: EditorViewState::Simple,
            hover_dialog: None,
            start_path: None,
        }
    }

    pub fn with_path(mut self, path: SPath) -> Self {
        let res = Self {
            start_path: Some(path),

            ..self
        };

        res
    }

    pub fn with_path_op(mut self, path_op: Option<SPath>) -> Self {
        let res = Self {
            start_path: path_op,
            ..self
        };

        res
    }

    // pub fn with_buffer(self, buffer: BufferSharedRef) -> Self {
    //     let navcomp_op: Option<NavCompRef> = if let Some(buffer_lock) = buffer.lock() {
    //         buffer_lock.get_path().map(|path| self.nav_comp_group.get_navcomp_for(path)).flatten()
    //     } else {
    //         error!("can't set navcomp - buffer lock aquisition failed");
    //         None
    //     };
    //
    //     let mut editor = self.editor;
    //     editor.internal_mut().set_buffer(buffer, navcomp_op);
    //
    //     EditorView {
    //         editor,
    //         ..self
    //     }
    // }

    pub fn get_buffer_ref(&self) -> &BufferSharedRef {
        self.editor.internal().get_buffer()
    }

    fn get_hover_rect(sc: SizeConstraint) -> Option<Rect> {
        sc.as_finite().map(|finite_sc| {
            if finite_sc >= XY::new(10, 8) {
                let margin = finite_sc / 10;
                let res = Rect::new(margin,
                                    finite_sc - margin * 2,
                );
                Some(res)
            } else {
                None
            }
        }).flatten()
    }

    /*
    This attempts to save current file, but in case that's not possible (filename unknown) proceeds to open_save_as_dialog() below
     */
    fn save_or_save_as(&mut self, buffer: &BufferState) {
        if let Some(ff) = buffer.get_path() {
            ff.overwrite_with_stream(&mut buffer.streaming_iterator(), false);
        } else {
            self.open_save_as_dialog_and_focus(buffer)
        }
    }

    fn open_save_as_dialog_and_focus(&mut self, buffer: &BufferState) {
        match self.state {
            EditorViewState::Simple => {}
            _ => {
                warn!("open_save_as_dialog in unexpected state");
            }
        }

        let save_file_dialog = SaveFileDialogWidget::new(
            self.providers.fsf().clone(),
        ).with_on_cancel(|_| {
            EditorViewMsg::OnSaveAsCancel.someboxed()
        }).with_on_save(|_, ff| {
            EditorViewMsg::OnSaveAsHit { ff }.someboxed()
        }).with_path(self.get_save_file_dialog_path(buffer));

        self.hover_dialog = Some(save_file_dialog);
        self.set_focused(self.get_hover_subwidget());
    }

    fn after_positive_save(&mut self, buffer_mut: &mut BufferState, path: &SPath) -> Option<MainViewMsg> {
        // setting the file path
        let set_path_result = self.set_file_name(buffer_mut, path);

        if set_path_result.path_changed {

            // updating the "save as dialog" starting position
            path.parent().map(|_| {
                self.start_path = Some(path.clone())
            }).unwrap_or_else(|| {
                error!("failed setting save_file_dialog starting position - most likely parent is outside fsf root");
            });


            Some(MainViewMsg::BufferChangedName { updated_identifier: set_path_result.document_id })
        } else {
            None
        }
    }

    /*
    This returns a (absolute) file path to be used with save_file_dialog. It can but does not have to
    contain filename part.
     */
    fn get_save_file_dialog_path(&self, buffer: &BufferState) -> SPath {
        if let Some(ff) = buffer.get_path() {
            return ff.clone();
        };

        if let Some(sp) = self.start_path.as_ref() {
            return sp.clone();
        }

        self.providers.fsf().root()
    }

    fn get_hover_subwidget(&self) -> SubwidgetPointer<Self> {
        SubwidgetPointer::new(Box::new(|w: &Self| {
            w.hover_dialog.as_ref().unwrap() // TODO
        }),
                              Box::new(|w: &mut Self| {
                                  w.hover_dialog.as_mut().unwrap() // TODO
                              }),
        )
    }

    fn hit_find_once(&mut self, buffer_mut: &mut BufferState) -> bool {
        let phrase = self.find_box.get_buffer().to_string();
        match self.editor.internal_mut().find_once(buffer_mut, &phrase) {
            Ok(changed) => changed,
            Err(_e) => {
                // TODO handle?
                error!("failed looking for {}", phrase);
                false
            }
        }
    }

    /*
    If we have selected item that matches current phrase, we replace it and do another lookup.
    Just lookup otherwise.
     */
    fn hit_replace_once(&mut self, buffer_mut: &mut BufferState) -> bool {
        let phrase = unpack_or!(self.get_pattern(), false ,"hit_replace_once with empty phrase - ignoring");
        let curr_text = buffer_mut.text();
        let editor_widget_id = self.editor.internal().id();
        let cursor_set = unpack_or!(curr_text.get_cursor_set(editor_widget_id), false, "no cursors for editor");

        if cursor_set.is_single() && curr_text.do_cursors_match_regex(editor_widget_id, &phrase) {
            let with_what = self.replace_box.get_buffer().to_string();
            let page_height = self.editor.internal().page_height() as usize;
            buffer_mut.apply_cem(
                CommonEditMsg::Block(with_what),
                editor_widget_id,
                page_height,
                Some(self.providers.clipboard()), //not really needed but why not
            );

            self.hit_find_once(buffer_mut);
            true
        } else {
            self.hit_find_once(buffer_mut)
        }
    }

    fn get_pattern(&self) -> Option<SearchPattern> {
        if self.find_box.is_empty() {
            None
        } else {
            Some(self.find_box.get_text().into())
        }
    }

    fn set_file_name(&mut self, buffer_mut: &mut BufferState, path: &SPath) -> SetFilePathResult {
        buffer_mut.set_file_path(Some(path.clone()))
    }

    pub fn get_path(&self) -> Option<SPath> {
        self.editor.internal().get_buffer().lock().map(|buffer_lock|
            buffer_lock.get_path().map(|c| c.clone())
        ).flatten()
    }

    pub fn get_internal_widget(&self) -> &EditorWidget {
        self.editor.internal()
    }
}

impl Widget for EditorView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn prelayout(&mut self) {
        self.complex_prelayout();
    }

    fn size(&self) -> XY {
        XY::new(10, 3) // TODO completely arbitrary
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        self.complex_layout(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let c = &self.providers.config().keyboard_config.editor;
        return match input_event {
            InputEvent::FocusUpdate(focus_update) => {
                if self.will_accept_focus_update(focus_update) {
                    Some(Box::new(EditorViewMsg::FocusUpdateMsg(focus_update)))
                } else {
                    None
                }
            }
            InputEvent::KeyInput(key) if key == c.save => {
                EditorViewMsg::Save.someboxed()
            }
            InputEvent::KeyInput(key) if key == c.save_as => {
                EditorViewMsg::SaveAs.someboxed()
            }
            InputEvent::KeyInput(key) if key == c.find => {
                EditorViewMsg::ToFind.someboxed()
            }
            InputEvent::KeyInput(key) if key == c.replace => {
                EditorViewMsg::ToFindReplace.someboxed()
            }
            InputEvent::KeyInput(key) if key == c.find => {
                EditorViewMsg::ToFind.someboxed()
            }
            InputEvent::KeyInput(key) if key == c.close_find_replace => {
                EditorViewMsg::ToSimple.someboxed()
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<EditorViewMsg>() {
            None => {
                debug!(target: "recursive_treat_views", "expected EditorViewMsg, got {:?}, passing through", msg);
                Some(msg) //passthrough
            }
            Some(msg) => {
                if let Some(mut buffer_lock) = self.editor.internal_mut().get_buffer().clone().lock_rw() {
                    match msg {
                        EditorViewMsg::Save => {
                            self.save_or_save_as(&buffer_lock);
                            None
                        }
                        EditorViewMsg::SaveAs => {
                            self.open_save_as_dialog_and_focus(&buffer_lock);
                            None
                        }
                        EditorViewMsg::OnSaveAsCancel => {
                            self.hover_dialog = None;
                            self.set_focused(subwidget!(Self.editor));
                            None
                        }
                        EditorViewMsg::OnSaveAsHit { ff } => {
                            // TODO handle errors and add test that
                            // TODO add test that checks if effects of after_positive_save are achieved
                            if ff.overwrite_with_stream(&mut buffer_lock.streaming_iterator(), false).is_ok() {
                                self.after_positive_save(&mut buffer_lock, ff);
                            }

                            self.hover_dialog = None;
                            self.set_focused(subwidget!(Self.editor));
                            None
                        }
                        EditorViewMsg::FocusUpdateMsg(focus_update) => {
                            // warn!("updating focus");
                            self.update_focus(*focus_update);
                            None
                        }
                        EditorViewMsg::ToSimple => {
                            self.state = EditorViewState::Simple;
                            self.find_box.clear();
                            self.replace_box.clear();
                            self.hover_dialog = None;
                            self.set_focused(subwidget!(Self.editor));
                            None
                        }
                        EditorViewMsg::ToFind => {
                            self.state = EditorViewState::Find;
                            self.replace_box.clear();
                            self.set_focused(subwidget!(Self.find_box));
                            None
                        }
                        EditorViewMsg::ToFindReplace => {
                            let old_state = self.state;
                            self.state = EditorViewState::FindReplace;

                            if old_state == EditorViewState::Find {
                                self.set_focused(subwidget!(Self.replace_box));
                            } else {
                                self.set_focused(subwidget!(Self.find_box));
                            }

                            None
                        }
                        EditorViewMsg::FindHit => {
                            if !self.find_box.is_empty() {
                                self.hit_find_once(&mut buffer_lock);
                            }
                            None
                        }
                        EditorViewMsg::ReplaceHit => {
                            if !self.find_box.is_empty() {
                                self.hit_replace_once(&mut buffer_lock);
                            }
                            None
                        }
                    }
                } else {
                    error!("failed to acquire buffer lock to update editor_view, swallowing msg {:?}", msg);
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let total_size = unpack_or_e!(
            self.display_state.as_ref().map(|ds| ds.total_size),
            (), "render before layout"
        );

        #[cfg(test)]
        output.emit_metadata(
            Metadata {
                id: self.wid,
                typename: self.typename().to_string(),
                rect: Rect::from_zero(total_size),
                focused,
            }
        );

        self.complex_render(theme, focused, output)
    }

    fn kite(&self) -> XY {
        XY::ZERO
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.complex_get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.complex_get_focused_mut()
    }
}

impl ComplexWidget for EditorView {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let editor_layout = LeafLayout::new(subwidget!(Self.editor)).boxed();
        let find_text_layout = LeafLayout::new(subwidget!(Self.find_label)).boxed();
        let find_box_layout = LeafLayout::new(subwidget!(Self.find_box)).boxed();
        let find_layout =
            SplitLayout::new(SplitDirection::Horizontal)
                .with(SplitRule::Fixed(PATTERN.width().try_into().unwrap()), find_text_layout)
                .with(SplitRule::Proportional(1.0), find_box_layout)
                .boxed();

        let replace_text_layout = LeafLayout::new(subwidget!(Self.replace_label)).boxed();
        let replace_box_layout = LeafLayout::new(subwidget!(Self.replace_box)).boxed();
        let replace_layout =
            SplitLayout::new(SplitDirection::Horizontal)
                .with(SplitRule::Fixed(REPLACE.width().try_into().unwrap()), replace_text_layout)
                .with(SplitRule::Proportional(1.0), replace_box_layout)
                .boxed();

        let background: Box<dyn Layout<Self>> = match &self.state {
            EditorViewState::Simple => {
                editor_layout
            }
            EditorViewState::Find => {
                SplitLayout::new(SplitDirection::Vertical)
                    .with(SplitRule::Proportional(1.0), editor_layout)
                    .with(SplitRule::Fixed(1), find_layout)
                    .boxed()
            }
            EditorViewState::FindReplace => {
                Box::new(SplitLayout::new(SplitDirection::Vertical)
                    .with(SplitRule::Proportional(1.0), editor_layout)
                    .with(SplitRule::Fixed(1), find_layout)
                    .with(SplitRule::Fixed(1), replace_layout)
                )
            }
        };

        if self.hover_dialog.is_none() {
            background
        } else {
            let hover = LeafLayout::new(SubwidgetPointer::new(
                Box::new(|s: &Self| {
                    s.hover_dialog.as_ref().unwrap()
                }),
                Box::new(|s: &mut Self| {
                    s.hover_dialog.as_mut().unwrap()
                }),
            )).boxed();

            HoverLayout::new(background,
                             hover,
                             Box::new(Self::get_hover_rect),
                             true,
            ).boxed()
        }
    }

    fn get_default_focused(&self) -> SubwidgetPointer<EditorView> {
        subwidget!(Self.editor)
    }

    fn set_display_state(&mut self, display_state: DisplayState<EditorView>) {
        self.display_state = Some(display_state)
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<EditorView>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}

impl HasInvariant for EditorView {
    fn check_invariant(&self) -> bool {
        self.editor.internal().check_invariant()
    }
}