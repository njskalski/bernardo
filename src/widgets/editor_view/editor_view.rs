use std::path::{Path, PathBuf};
use std::rc::Rc;
use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;
use crate::{AnyMsg, ConfigRef, FsfRef, InputEvent, Output, SizeConstraint, Theme, TreeSitterWrapper, Widget};
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::focus_group::FocusUpdate;
use crate::experiments::regex_search::FindError;
use crate::io::sub_output::SubOutput;
use crate::layout::display_state::GenericDisplayState;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::search_pattern::SearchPattern;
use crate::primitives::xy;
use crate::primitives::xy::XY;
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::editor_view::msg::EditorViewMsg;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
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

pub struct EditorView {
    wid: WID,

    display_state: Option<GenericDisplayState>,

    editor: WithScroll<EditorWidget>,
    find_box: EditBoxWidget,
    find_label: TextWidget,
    replace_box: EditBoxWidget,
    replace_label: TextWidget,

    /*
    resist the urge to remove fsf from editor. It's used to facilitate "save as dialog".
    You CAN be working on two different filesystems at the same time, and save as dialog is specific to it.

    One thing to address is: "what if I have file from filesystem A, and I want to "save as" to B?". But that's beyond MVP, so I don't think about it now.
     */
    fsf: FsfRef,
    config: ConfigRef,
    // this is necessary since there are multiple clipboard receivers within this object.
    clipboard: ClipboardRef,

    state: EditorViewState,
    hover_dialog: Option<SaveFileDialogWidget>,

    /*
    This represents "where the save as dialog should start", but only in case the file_front on buffer_state is None.
    If none, we'll use the fsf root.
    See get_save_file_dialog_path for details.
     */
    start_path: Option<Rc<PathBuf>>,

    // this is a workaround to set focus into something not visible yet in next frame.
    deferred_focus: Option<WID>,
}

impl EditorView {
    pub fn new(
        config: ConfigRef,
        tree_sitter: Rc<TreeSitterWrapper>,
        fsf: FsfRef,
        clipboard: ClipboardRef,
    ) -> Self {
        let editor = EditorWidget::new(config.clone(),
                                       tree_sitter,
                                       fsf.clone(),
                                       clipboard.clone());

        let find_label = TextWidget::new(Box::new(PATTERN));
        let replace_label = TextWidget::new(Box::new(REPLACE));

        let find_box = EditBoxWidget::new().with_on_hit(|w| {
            EditorViewMsg::FindHit.someboxed()
        });
        let replace_box = EditBoxWidget::new().with_on_hit(|_| {
            EditorViewMsg::ReplaceHit.someboxed()
        });

        EditorView {
            wid: get_new_widget_id(),
            display_state: None,
            editor: WithScroll::new(editor, ScrollDirection::Vertical).with_line_no(),
            find_box,
            find_label,
            replace_box,
            replace_label,
            fsf,
            config,
            clipboard,
            state: EditorViewState::Simple,
            hover_dialog: None,
            start_path: None,
            deferred_focus: None,
        }
    }

    pub fn with_path(self, path: Rc<PathBuf>) -> Self {
        Self {
            start_path: Some(path),
            ..self
        }
    }

    pub fn with_path_op(self, path_op: Option<Rc<PathBuf>>) -> Self {
        Self {
            start_path: path_op,
            ..self
        }
    }

    pub fn with_buffer(self, buffer: BufferState) -> Self {
        let editor = self.editor.mutate_internal(move |b| b.with_buffer(buffer));

        EditorView {
            editor,
            ..self
        }
    }

    fn get_hover_rect(max_size: XY) -> Rect {
        let margin = max_size / 10;
        Rect::new(margin,
                  max_size - margin * 2,
        )
    }

    fn internal_layout(&mut self, size: XY) -> Vec<WidgetIdRect> {
        let mut editor_layout = LeafLayout::new(&mut self.editor);
        let mut find_text_layout = LeafLayout::new(&mut self.find_label);
        let mut find_box_layout = LeafLayout::new(&mut self.find_box);
        let mut find_layout =
            SplitLayout::new(SplitDirection::Horizontal)
                .with(SplitRule::Fixed(PATTERN.width_cjk()), &mut find_text_layout)
                .with(SplitRule::Proportional(1.0), &mut find_box_layout);

        let mut replace_box_layout = LeafLayout::new(&mut self.replace_box);
        let mut replace_text_layout = LeafLayout::new(&mut self.replace_label);
        let mut replace_layout =
            SplitLayout::new(SplitDirection::Horizontal)
                .with(SplitRule::Fixed(REPLACE.width_cjk()), &mut replace_text_layout)
                .with(SplitRule::Proportional(1.0), &mut replace_box_layout);


        let mut background: Box<dyn Layout> = match &mut self.state {
            EditorViewState::Simple => {
                Box::new(editor_layout)
            }
            EditorViewState::Find => {
                Box::new(SplitLayout::new(SplitDirection::Vertical)
                    .with(SplitRule::Proportional(1.0), &mut editor_layout)
                    .with(SplitRule::Fixed(1), &mut find_layout)
                )
            }
            EditorViewState::FindReplace => {
                Box::new(SplitLayout::new(SplitDirection::Vertical)
                    .with(SplitRule::Proportional(1.0), &mut editor_layout)
                    .with(SplitRule::Fixed(1), &mut find_layout)
                    .with(SplitRule::Fixed(1), &mut replace_layout)
                )
            }
        };

        match self.hover_dialog.as_mut() {
            None => background.calc_sizes(size),
            Some(dialog) => {
                let rect = Self::get_hover_rect(size);
                HoverLayout::new(&mut *background,
                                 &mut LeafLayout::new(dialog),
                                 rect,
                ).calc_sizes(size)
            }
        }
    }

    /*
    This attempts to save current file, but in case that's not possible (filename unknown) proceeds to open_save_as_dialog() below
     */
    fn save_or_save_as(&mut self) {
        let buffer = self.editor.internal().buffer();

        if let Some(ff) = buffer.get_file_front() {
            ff.overwrite_with(buffer);
        } else {
            self.open_save_as_dialog()
        }
    }

    fn open_save_as_dialog(&mut self) {
        match self.state {
            EditorViewState::Simple => {}
            _ => {
                warn!("open_save_as_dialog in unexpected state");
            }
        }

        let save_file_dialog = SaveFileDialogWidget::new(
            self.fsf.clone(),
        ).with_on_cancel(|_| {
            EditorViewMsg::OnSaveAsCancel.someboxed()
        }).with_on_save(|_, ff| {
            EditorViewMsg::OnSaveAsHit { ff }.someboxed()
        }).with_path(self.get_save_file_dialog_path());

        self.hover_dialog = Some(save_file_dialog);
    }

    fn positively_save_raw(&mut self, path: &Path) {
        let ff = match self.fsf.get_item(path) {
            None => {
                error!("attempted saving beyond root path");
                return;
            }
            Some(p) => p,
        };

        // setting the file path
        let buffer = self.editor.internal_mut().buffer_mut();
        buffer.set_file_front(Some(ff.clone()));

        // updating the "save as dialog" starting position
        ff.parent().map(|_f| {
            self.start_path = Some(ff.path_rc().clone())
        }).unwrap_or_else(|| {
            error!("failed setting save_as_dialog starting position - most likely parent is outside fsf root");
        });
    }

    /*
    This returns a (absolute) file path to be used with save_file_dialog. It can but does not have to
    contain filename part.
     */
    fn get_save_file_dialog_path(&self) -> &Rc<PathBuf> {
        let buffer = self.editor.internal().buffer();
        if let Some(ff) = buffer.get_file_front() {
            return ff.path_rc();
        };

        if let Some(sp) = self.start_path.as_ref() {
            return sp;
        }

        self.fsf.get_root_path()
    }

    pub fn buffer_state(&self) -> &BufferState {
        self.editor.internal().buffer_state()
    }

    fn hit_find_once(&mut self) -> bool {
        let phrase = self.find_box.get_text().to_string();
        match self.editor.internal_mut().find_once(&phrase) {
            Ok(changed) => changed,
            Err(e) => {
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
    fn hit_replace_once(&mut self) -> bool {
        let phrase = match self.get_pattern() {
            Some(p) => p,
            None => {
                debug!("hit_replace_once with empty phrase - ignoring");
                return false;
            }
        };

        let curr_text = self.editor.internal().buffer_state().text();
        if curr_text.cursor_set.is_single() && curr_text.do_cursors_match_regex(&phrase) {
            let with_what = self.replace_box.get_text().to_string();
            let page_height = self.editor.internal().page_height() as usize;
            let bf = self.editor.internal_mut().buffer_state_mut();
            bf.apply_cem(
                CommonEditMsg::Block(with_what),
                page_height,
                Some(&self.clipboard), //not really needed but why not
            );

            self.hit_find_once();
            true
        } else {
            self.hit_find_once()
        }
    }

    fn set_deferred_focus(&mut self, wid: WID) {
        if self.deferred_focus.is_some() {
            warn!("overriding deferred focus before it was flushed!")
        }
        self.deferred_focus = Some(self.find_box.id())
    }

    /*
    This checks, if next hit to replace does an actual replace and next find, or just first find.
     */

    fn get_pattern(&self) -> Option<SearchPattern> {
        if self.find_box.is_empty() {
            None
        } else {
            Some(self.find_box.get_text().into())
        }
    }
}

impl Widget for EditorView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "editor_view"
    }

    fn min_size(&self) -> XY {
        XY::new(20, 8) // TODO completely arbitrary
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        //This is copied from SaveFileDialog, and should be replaced when focus is removed from widgets.
        let max_size = sc.visible_hint().size;
        let res_sizes = self.internal_layout(max_size);

        let focus_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());

        let mut ds = GenericDisplayState::new(max_size, res_sizes);

        match &self.state {
            EditorViewState::Find => {
                ds.focus_group.add_edge(self.editor.id(), FocusUpdate::Down, self.find_box.id());
                ds.focus_group.add_edge(self.find_box.id(), FocusUpdate::Up, self.editor.id());
            }
            EditorViewState::FindReplace => {
                ds.focus_group.add_edge(self.editor.id(), FocusUpdate::Down, self.find_box.id());
                ds.focus_group.add_edge(self.find_box.id(), FocusUpdate::Up, self.editor.id());

                ds.focus_group.add_edge(self.find_box.id(), FocusUpdate::Down, self.replace_box.id());
                ds.focus_group.add_edge(self.replace_box.id(), FocusUpdate::Up, self.find_box.id());
            }
            _ => {}
        }

        if let Some(wid) = &self.deferred_focus {
            if !ds.focus_group.set_focused(*wid) {
                error!("failed to set a deferred focus");
            }
            self.deferred_focus = None;
        } else if let Some(wid) = focus_op {
            if !ds.focus_group.set_focused(wid) {
                error!("failed to reset focus");
            }
        }

        self.display_state = Some(ds);
        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let c = &self.config.keyboard_config.editor;
        return match input_event {
            InputEvent::FocusUpdate(focus_update) => {
                let can_update = self.display_state.as_ref().map(|ds| {
                    ds.focus_group().can_update_focus(focus_update)
                }).unwrap_or(false);

                if can_update {
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
                warn!("expecetd EditorViewMsg, got {:?}", msg);
                None
            }
            Some(msg) => match msg {
                EditorViewMsg::Save => {
                    self.save_or_save_as();
                    None
                }
                EditorViewMsg::SaveAs => {
                    self.open_save_as_dialog();
                    None
                }
                EditorViewMsg::OnSaveAsCancel => {
                    self.hover_dialog = None;
                    None
                }
                EditorViewMsg::OnSaveAsHit { ff } => {
                    // TODO handle errors
                    ff.overwrite_with(self.editor.internal().buffer());
                    self.hover_dialog = None;
                    None
                }
                EditorViewMsg::FocusUpdateMsg(focus_update) => {
                    // warn!("updating focus");
                    self.display_state.as_mut().map(
                        |ds| {
                            if !ds.focus_group.update_focus(*focus_update) {
                                warn!("focus update accepted but failed");
                            }
                            None
                        }
                    ).unwrap_or_else(|| {
                        error!("failed retrieving display_state");
                        None
                    })
                }
                EditorViewMsg::ToSimple => {
                    self.state = EditorViewState::Simple;
                    self.find_box.clear();
                    self.replace_box.clear();
                    self.hover_dialog = None;
                    self.set_focused(self.editor.id());
                    None
                }
                EditorViewMsg::ToFind => {
                    self.state = EditorViewState::Find;
                    self.replace_box.clear();
                    self.set_deferred_focus(self.find_box.id());
                    None
                }
                EditorViewMsg::ToFindReplace => {
                    let old_state = self.state;
                    self.state = EditorViewState::FindReplace;

                    if old_state == EditorViewState::Find {
                        self.set_deferred_focus(self.replace_box.id());
                    } else {
                        self.set_deferred_focus(self.find_box.id());
                    }

                    None
                }
                EditorViewMsg::FindHit => {
                    if !self.find_box.is_empty() {
                        self.hit_find_once();
                    }
                    None
                }
                EditorViewMsg::ReplaceHit => {
                    if !self.find_box.is_empty() {
                        self.hit_replace_once();
                    }
                    None
                }
            }
        };
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        if let Some(hd) = &self.hover_dialog {
            return Some(hd);
        }

        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(|wid| self.get_subwidget(wid)).flatten()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        // if let Some(hd) = &mut self.hover_dialog {
        //     return Some(hd as &mut dyn Widget);
        // }
        // it has to be written badly, because otherwise borrowchecker is sad.
        if self.hover_dialog.is_some() {
            return self.hover_dialog.as_mut().map(|f| f as &mut dyn Widget);
        }

        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(move |wid| self.get_subwidget_mut(wid)).flatten()
    }

    fn set_focused(&mut self, wid: WID) -> bool {
        if let Some(ds) = &mut self.display_state {
            ds.focus_group_mut().set_focused(wid)
        } else {
            error!("set_focused with no display state.");
            false
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        if let Some(cached_sizes) = &self.display_state {
            let focused_child_id_op = self.get_focused().map(|f| f.id());
            for wir in &cached_sizes.widget_sizes {
                match self.get_subwidget(wir.wid) {
                    Some(widget) => {
                        let sub_output = &mut SubOutput::new(output, wir.rect);
                        widget.render(theme,
                                      focused && focused_child_id_op == Some(wir.wid),
                                      sub_output,
                        );
                    }
                    None => {
                        warn!("subwidget {} not found!", wir.wid);
                    }
                }
            }
        } else {
            error!("render absent display state");
        }
    }

    fn anchor(&self) -> XY {
        self.editor.internal().anchor()
    }

    fn subwidgets_mut(&mut self) -> Box<dyn Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        let mut res: Vec<&mut dyn Widget> = vec![&mut self.editor];

        match &mut self.state {
            EditorViewState::Simple => {}
            EditorViewState::Find => {
                res.push(&mut self.find_box);
                res.push(&mut self.find_label);
            }
            EditorViewState::FindReplace => {
                res.push(&mut self.find_box);
                res.push(&mut self.find_label);
                res.push(&mut self.replace_box);
                res.push(&mut self.replace_label);
            }
        };

        if let Some(hd) = &mut self.hover_dialog {
            res.push(hd);
        }

        Box::new(res.into_iter())
    }

    fn subwidgets(&self) -> Box<dyn Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        let mut res: Vec<&dyn Widget> = vec![&self.editor];

        match &self.state {
            EditorViewState::Simple => {}
            EditorViewState::Find => {
                res.push(&self.find_box);
                res.push(&self.find_label);
            }
            EditorViewState::FindReplace => {
                res.push(&self.find_box);
                res.push(&self.find_label);
                res.push(&self.replace_box);
                res.push(&self.replace_label);
            }
        };

        if let Some(hd) = &self.hover_dialog {
            res.push(hd);
        }


        Box::new(res.into_iter())
    }
}
