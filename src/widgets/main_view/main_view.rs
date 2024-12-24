use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::{format, Display};
use std::sync::Arc;

use log::{debug, error, warn};
use uuid::Uuid;

use crate::config::theme::Theme;
use crate::cursor::cursor::Cursor;
use crate::experiments::buffer_register::OpenResult;
use crate::experiments::filename_to_language::filename_to_language;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::path::SPath;
use crate::fs::read_error::ReadError;
use crate::gladius::msg::GladiusMsg;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::common_query::CommonQuery;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::symbol_usage::SymbolUsage;
use crate::primitives::tree::tree_node::TreeNode;
use crate::primitives::xy::XY;
use crate::promise::streaming_promise::StreamingPromise;
use crate::text::text_buffer::TextBuffer;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::context_bar_item::ContextBarItem;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::code_results_view::full_text_search_code_results_provider::FullTextSearchCodeResultsProvider;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::file_tree_view::file_tree_view::FileTreeViewWidget;
use crate::widgets::find_in_files_widget::find_in_files_widget::FindInFilesWidget;
use crate::widgets::main_view::display::MainViewDisplay;
use crate::widgets::main_view::focus_path_widget::FocusPathWidget;
use crate::widgets::main_view::fuzzy_file_search_widget::FuzzyFileSearchWidget;
use crate::widgets::main_view::fuzzy_screens_list_widget::{get_fuzzy_screen_list, FuzzyScreensList};
use crate::widgets::main_view::main_context_menu::{aggregate_actions, get_focus_path_w, MainContextMenuWidget};
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::main_view::util;
use crate::widgets::main_view::util::get_focus_path;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::spath_tree_view_node::FileTreeNode;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;
use crate::{subwidget, unpack_or, unpack_or_e};

pub type BufferId = Uuid;

pub enum HoverItem {
    // used in fuzzy buffer list
    FuzzySearch(FuzzyScreensList),
    // used in fuzzy file list
    FuzzySearch2(FuzzyFileSearchWidget),

    // search in files
    SearchInFiles(FindInFilesWidget),

    // Context menu
    ContextMain {
        anchor: XY,
        widget: MainContextMenuWidget,

        // I need this to put focus back where it was on CLOSING of context menu.
        old_focus: SubwidgetPointer<MainView>,
    },
}

// TODO start indexing documents with DocumentIdentifier as opposed to usize

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct DocumentIdentifier {
    pub buffer_id: BufferId,
    pub file_path: Option<SPath>,
}

impl Display for DocumentIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.file_path {
            None => {
                write!(f, "[unnamed] #{}", self.buffer_id)
            }
            Some(sp) => {
                write!(f, "{} #{}", sp.label(), self.buffer_id)
            }
        }
    }
}

impl DocumentIdentifier {
    pub fn new_unique() -> DocumentIdentifier {
        DocumentIdentifier {
            buffer_id: Uuid::new_v4(),
            file_path: None,
        }
    }

    pub fn with_file_path(self, ff: SPath) -> Self {
        Self {
            file_path: Some(ff),
            ..self
        }
    }

    pub fn label(&self) -> String {
        match &self.file_path {
            None => {
                format!("[unnamed]")
            }
            Some(sp) => {
                format!("{}", sp.label())
            }
        }
    }
}

pub struct MainView {
    wid: WID,
    providers: Providers,
    /*
    I use a simplified "display state" model, not the GenericFocusGroup approach, to see how much effort the Generic one saves.
    caveat: whenever focusing on editor, make sure to set curr_editor_index as well. It's a temporary solution, so I don't wrap it.
     */
    display_state: Option<DisplayState<MainView>>,

    tree_widget: FileTreeViewWidget,
    no_editor: NoEditorWidget,
    displays: Vec<MainViewDisplay>,
    display_idx: usize,

    status_bar: FocusPathWidget,

    hover: Option<HoverItem>,
}

impl MainView {
    pub const MIN_SIZE: XY = XY::new(32, 10);
    pub const TYPENAME: &'static str = "main_view";

    pub fn create_new_display_for_code_results(&mut self, data_provider: Box<dyn CodeResultsProvider>) -> Result<usize, ()> {
        self.displays.push(MainViewDisplay::ResultsView(CodeResultsView::new(
            self.providers.clone(),
            data_provider,
        )));

        let res = self.displays.len() - 1;

        Ok(res)
    }

    pub fn create_new_editor_for_file(&mut self, ff: &SPath) -> Result<usize, ReadError> {
        // TODO this should return some other error, but they are swallowed anyway
        let mut register_lock = unpack_or_e!(
            self.providers.buffer_register().try_write().ok(),
            Err(ReadError::FileNotFound),
            "failed to lock register"
        );
        let OpenResult {
            buffer_shared_ref,
            opened: _,
        } = register_lock.open_file(&self.providers, ff);
        let buffer_shared_ref = buffer_shared_ref?;

        if let Some(mut buffer_lock) = buffer_shared_ref.lock_rw() {
            buffer_lock.set_lang(filename_to_language(&ff));
        }

        self.displays.push(MainViewDisplay::Editor(
            EditorView::new(self.providers.clone(), buffer_shared_ref).with_path_op(ff.parent()),
        ));

        let res = self.displays.len() - 1;

        Ok(res)
    }

    fn get_curr_display_ptr(&self) -> SubwidgetPointer<Self> {
        if self.display_idx >= self.displays.len() {
            if self.display_idx > 0 {
                error!("current editor points further than number opened editors!");
                return subwidget!(Self.tree_widget);
            }

            subwidget!(Self.no_editor)
        } else {
            let idx1 = self.display_idx;
            let idx2 = self.display_idx;
            SubwidgetPointer::new(
                Box::new(move |s: &Self| s.displays.get(idx1).map(|w| w.get_widget()).unwrap_or(&s.no_editor)),
                Box::new(move |s: &mut Self| s.displays.get_mut(idx2).map(|w| w.get_widget_mut()).unwrap_or(&mut s.no_editor)),
            )
        }
    }

    fn get_editor_idx_for(&self, ff: &SPath) -> Option<usize> {
        let register = unpack_or_e!(self.providers.buffer_register().try_read().ok(), None, "failed locking register");
        let buffer_shared_ref = unpack_or!(register.get_buffer_ref_from_path(ff), None, "no buffer for path");

        for (idx, display) in self.displays.iter().enumerate() {
            match display {
                MainViewDisplay::Editor(editor) => {
                    if *editor.get_buffer_ref() == buffer_shared_ref {
                        return Some(idx);
                    }
                }
                MainViewDisplay::ResultsView(_) => {}
            }
        }
        None
    }

    fn get_hover_rect(screenspace: Screenspace) -> Option<Rect> {
        let output_size = screenspace.output_size();
        if output_size >= XY::new(10, 8) {
            let margin = output_size / 10;
            let res = Rect::new(margin, output_size - margin * 2);
            Some(res)
        } else {
            None
        }
    }

    pub fn new(providers: Providers) -> MainView {
        let root = providers.fsf().root();

        MainView {
            wid: get_new_widget_id(),
            providers: providers.clone(),
            display_state: None,
            tree_widget: FileTreeViewWidget::new(providers.config().clone(), root),
            displays: Vec::new(),
            no_editor: NoEditorWidget::default(),
            display_idx: 0,
            status_bar: FocusPathWidget::new(),
            hover: None,
        }
    }

    fn open_find_in_files(&mut self) {
        if self.hover.is_some() {
            debug!("ignoring 'open find everywhere', because there is already a hover");
            return;
        }

        self.hover = Some(HoverItem::SearchInFiles(
            FindInFilesWidget::new(self.providers.fsf().root())
                .with_on_hit(Some(Box::new(|widget| {
                    MainViewMsg::FindInFilesQuery {
                        root_dir: widget.root().clone(),
                        query: widget.get_query(),
                        filter_op: widget.get_filter(),
                    }
                    .someboxed()
                })))
                .with_on_cancel(Some(Box::new(|_| MainViewMsg::CloseHover.someboxed()))),
        ));
        self.set_focus_to_hover();
    }

    // TODO add filtering
    // TODO add checkbox for "ignore git"
    fn handle_open_find_in_files(&mut self, root_dir: SPath, query: String, filter_op: Option<String>) {
        self.hover = None;
        let desc = format!("full text search of '{}' ", query);

        let promise: Box<dyn StreamingPromise<SymbolUsage>> = unpack_or_e!(
            root_dir.start_full_text_search(CommonQuery::String(query), true).ok(),
            (),
            "failed to start full text search"
        );
        let idx = unpack_or_e!(
            self.create_new_display_for_code_results(FullTextSearchCodeResultsProvider::new(Arc::new(desc), promise).boxed())
                .ok(),
            (),
            "failed creating full_text_search view"
        );
        self.display_idx = idx;
        self.set_focus_to_default();
    }

    fn open_empty_editor_and_focus(&mut self) {
        let buffer = if let Ok(mut buffer_register) = self.providers.buffer_register().try_write() {
            buffer_register.open_new_file(&self.providers)
        } else {
            error!("failed to acquire register lock");
            return;
        };

        self.displays
            .push(MainViewDisplay::Editor(EditorView::new(self.providers.clone(), buffer)));

        self.display_idx = self.displays.len() - 1;
        self.set_focus_to_default();
    }

    pub fn open_file_with_path_and_focus(&mut self, ff: SPath) -> bool {
        debug!("opening file {:?}", ff);

        if let Some(idx) = self.get_editor_idx_for(&ff) {
            self.display_idx = idx;
            self.set_focused(self.get_default_focused());
            true
        } else {
            self.create_new_editor_for_file(&ff)
                .map(|idx| {
                    self.display_idx = idx;
                    self.set_focused(self.get_default_focused());
                })
                .is_ok()
        }
    }

    fn open_fuzzy_buffer_list_and_focus(&mut self) {
        let len = self.displays.len();

        let mut fsl = FuzzyScreensList::new(self.providers.clone(), get_fuzzy_screen_list(&self.displays, self.display_idx))
            .with_on_hit(Box::new(move |w| {
                if *w.get_highlighted().1.id() >= len {
                    None
                } else {
                    MainViewMsg::FocusOnDisplay {
                        display_idx: *w.get_highlighted().1.id(),
                    }
                    .someboxed()
                }
            }))
            .with_on_close(Box::new(|_| MainViewMsg::CloseHover.someboxed()))
            .with_expanded_root();

        // I use "len +1" and "len +2" for "buffers" and "searches"
        fsl.tree_view_mut().set_expanded(len + 1, true);
        fsl.tree_view_mut().set_expanded(len + 2, true);

        self.hover = Some(HoverItem::FuzzySearch(fsl));
        self.set_focus_to_hover();
    }

    fn open_fuzzy_search_in_files_and_focus(&mut self) {
        self.hover = Some(HoverItem::FuzzySearch2(
            FuzzyFileSearchWidget::new(self.providers.clone(), FileTreeNode::new(self.providers.fsf().root().clone()))
                .with_on_hit(Box::new(|w| {
                    let spath = w.get_highlighted().1.spath().clone();
                    MainViewMsg::OpenFileBySpath { spath }.someboxed()
                }))
                .with_on_close(Box::new(|_| MainViewMsg::CloseHover.someboxed()))
                .with_expanded_root(),
        ));
        self.set_focus_to_hover();
    }

    fn get_opened_views_for_document_id(
        &self,
        document_identifier: DocumentIdentifier,
    ) -> impl Iterator<Item = (usize, &MainViewDisplay)> + '_ {
        self.displays.iter().enumerate().filter_map(move |(idx, item)| match item {
            MainViewDisplay::ResultsView(_) => None,
            MainViewDisplay::Editor(editor) => {
                if editor.get_buffer_ref().document_identifier() == &document_identifier {
                    Some((idx, item))
                } else {
                    None
                }
            }
        })
    }

    fn get_hover_subwidget(&self) -> Option<SubwidgetPointer<Self>> {
        if self.hover.is_some() {
            Some(SubwidgetPointer::new(
                Box::new(|mv: &MainView| {
                    if mv.hover.is_some() {
                        match mv.hover.as_ref().unwrap() {
                            HoverItem::FuzzySearch(fs) => fs as &dyn Widget,
                            HoverItem::FuzzySearch2(fs) => fs as &dyn Widget,
                            HoverItem::SearchInFiles(fs) => fs as &dyn Widget,
                            HoverItem::ContextMain { anchor, widget, old_focus } => widget as &dyn Widget,
                        }
                    } else {
                        error!("no hover found, this subwidget pointer should have been overriden by now.");
                        let sw = mv.get_default_focused().clone();
                        sw.get(mv)
                    }
                }),
                Box::new(|mv: &mut MainView| {
                    if mv.hover.is_some() {
                        match mv.hover.as_mut().unwrap() {
                            HoverItem::FuzzySearch(fs) => fs as &mut dyn Widget,
                            HoverItem::FuzzySearch2(fs) => fs as &mut dyn Widget,
                            HoverItem::SearchInFiles(fs) => fs as &mut dyn Widget,
                            HoverItem::ContextMain { anchor, widget, old_focus } => widget as &mut dyn Widget,
                        }
                    } else {
                        error!("no hover found, this subwidget pointer should have been overriden by now.");
                        let sw = mv.get_default_focused().clone();
                        sw.get_mut(mv)
                    }
                }),
            ))
        } else {
            None
        }
    }

    // opens new document or brings old one to forefront (TODO this is temporary)
    // this entire method should be remade, it's here just to facilitate test that "hitting enter on
    // go to definition goes somewhere"
    fn open_document_and_focus(&mut self, document_identifier: DocumentIdentifier, position_op: Option<Cursor>) -> bool {
        let idx_op = self
            .get_opened_views_for_document_id(document_identifier.clone())
            .next()
            .map(|pair| pair.0);

        let succeeded = if let Some(idx) = idx_op {
            self.display_idx = idx;
            let widget_getter = self.get_default_focused();
            self.set_focused(widget_getter.clone());

            true
        } else {
            if let Some(path) = document_identifier.file_path {
                self.open_file_with_path_and_focus(path)
            } else {
                error!("no path for document {:?} provided - cannot open.", document_identifier);
                false
            }
        };

        if !succeeded {
            return false;
        }

        if let Some(position) = position_op {
            let cfe = unpack_or_e!(self.get_currently_focused_editor_view_mut(), false, "no editor focused");
            cfe.override_cursor_set(position.as_cursor_set());
        }

        true
    }

    fn get_currently_focused_editor_view_mut(&mut self) -> Option<&mut EditorView> {
        // let picker = unpack_or_e!(self.get_focused_mut(), None, "get_focused_mut() == None");
        debug_assert!(self.display_idx < self.displays.len());

        self.displays.get_mut(self.display_idx).map(|item| item.as_editor_mut()).flatten()
    }

    fn set_focus_to_default(&mut self) {
        let ptr = self.get_curr_display_ptr();
        self.set_focused(ptr);
    }

    fn set_focus_to_hover(&mut self) {
        if let Some(subwidget_ptr) = self.get_hover_subwidget() {
            self.set_focused(subwidget_ptr);
        } else {
            error!("failed to set focus to hover - no hover found! Setting to default.");
            self.set_focus_to_default();
        }
    }

    fn get_current_focus(&self) -> SubwidgetPointer<Self> {
        if let Some(item) = self.display_state.as_ref().map(|item| item.focused.clone()) {
            item
        } else {
            error!("get_current_focus before display_state is set!");
            self.get_curr_display_ptr()
        }
    }

    pub fn with_empty_editor(mut self) -> Self {
        self.open_empty_editor_and_focus();
        self
    }

    pub fn open_main_context_menu_and_focus(&mut self) {
        // TODO add checks for hover being empty or whatever
        let options = aggregate_actions(self);
        let final_kite = self.kite(); // kite is recursive now
        let item = ContextBarItem::new_internal_node(Cow::Borrowed("†"), options);
        let mut widget =
            MainContextMenuWidget::new(self.providers.clone(), item).with_on_close(Box::new(|_| MainViewMsg::CloseHover.someboxed()));

        let old_focus = self.get_current_focus();

        {
            widget.set_on_shortcut_hit(Box::new(move |widget, item| -> Option<Box<dyn AnyMsg>> {
                let depth = item.get_depth();
                let msg = item.on_hit()?;

                MainViewMsg::ContextMenuHit { msg: Some(msg), depth }.someboxed()
            }));
        }

        widget.tree_view_mut().expand_all_internal_nodes();

        if !self.providers.config().learning_mode {
            widget = widget.with_on_hit(Box::new(move |widget| -> Option<Box<dyn AnyMsg>> {
                let highlighted = widget.get_highlighted();
                let depth = highlighted.1.get_depth();
                let msg = highlighted.1.on_hit()?;

                MainViewMsg::ContextMenuHit { msg: Some(msg), depth }.someboxed()
            }));
        }

        self.hover = Some(HoverItem::ContextMain {
            anchor: final_kite,
            widget,
            old_focus,
        });
        self.set_focus_to_hover();
    }

    // TODO this piece of code would be great to unify with act_on, but I have no idea how to do it now.
    fn context_menu_process_hit(&mut self, msg: Box<dyn AnyMsg>, depth: usize) -> Option<Box<dyn AnyMsg>> {
        fn rec_process_msg(widget: &mut dyn Widget, depth: usize, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
            if depth == 0 {
                widget.update(msg)
            } else {
                if let Some(child) = widget.get_focused_mut() {
                    let result_op = rec_process_msg(child, depth - 1, msg);

                    if let Some(new_msg) = result_op {
                        widget.update(new_msg)
                    } else {
                        None
                    }
                } else {
                    error!("failed reaching expected focus path depth! Dropping msg {:?}", msg);
                    None
                }
            }
        }

        rec_process_msg(self, depth, msg)
    }

    fn do_prune_unchanged_buffers(&mut self) -> bool {
        let mut buffer_register = unpack_or_e!(
            self.providers.buffer_register().write().ok(),
            false,
            "failed to lock buffer register"
        );

        let mut buffers_to_close: HashSet<DocumentIdentifier> = Default::default();

        for (id, bf) in buffer_register.iter() {
            if let Some(buffer) = bf.lock() {
                if buffer.is_saved() {
                    buffers_to_close.insert(id.clone());
                }
            }
        }

        let mut indices_to_remove: Vec<usize> = Default::default();
        for (idx, disp) in self.displays.iter().enumerate() {
            if let Some(editor) = disp.as_editor() {
                let di = editor.get_buffer_ref().document_identifier();
                if buffers_to_close.contains(di) {
                    indices_to_remove.push(idx);
                }
            }
        }

        // this is potentially O(n^2), we need to fix it

        for idx in indices_to_remove.iter().rev() {
            if *idx < self.display_idx {
                self.display_idx -= 1;
            }

            self.displays.remove(*idx);
        }

        let mut success: bool = false;

        for buffer_id in buffers_to_close {
            success = success && buffer_register.close_buffer(&buffer_id);
        }

        success
    }
}

impl Widget for MainView {
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

    fn prelayout(&mut self) {
        self.complex_prelayout();

        let focus_path = get_focus_path(self);
        self.status_bar.set_focus_path(focus_path);
    }

    fn full_size(&self) -> XY {
        // TODO delegate to complex_layout?
        Self::MIN_SIZE
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("main_view.on_input {:?}", input_event);

        let config = self.providers.config();

        match input_event {
            InputEvent::FocusUpdate(focus_update) if self.will_accept_focus_update(focus_update) => {
                MainViewMsg::FocusUpdateMsg(focus_update).someboxed()
            }
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.close => MainViewMsg::QuitGladius.someboxed(),
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.new_buffer => MainViewMsg::OpenNewFile.someboxed(),
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.fuzzy_file => MainViewMsg::OpenFuzzyFiles.someboxed(),
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.browse_buffers => {
                if self.displays.is_empty() {
                    debug!("ignoring browse_buffers request - no displays open.");
                    None
                } else {
                    MainViewMsg::OpenChooseDisplay.someboxed()
                }
            }
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.find_in_files => MainViewMsg::OpenFindInFiles.someboxed(),
            InputEvent::EverythingBarTrigger => MainViewMsg::OpenContextMenu.someboxed(),
            // InputEvent::KeyInput(key) if key == config.keyboard_config.global.everything_bar => MainViewMsg::OpenContextMenu.someboxed(),
            _ => {
                debug!("input {:?} NOT consumed", input_event);
                None
            }
        }
    }

    fn update(&mut self, mut msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("main_view.update {:?}", msg);

        if let Some(main_view_msg) = msg.as_msg_mut::<MainViewMsg>() {
            return match main_view_msg {
                MainViewMsg::FocusUpdateMsg(focus_update) => {
                    if !self.update_focus(*focus_update) {
                        error!("failed to accept focus update")
                    }

                    None
                }
                MainViewMsg::TreeExpandedFlip { .. } => None,
                MainViewMsg::TreeSelected { item } => {
                    if !self.open_file_with_path_and_focus(item.clone()) {
                        error!("failed open_file");
                    }

                    None
                }
                MainViewMsg::OpenFuzzyFiles => {
                    self.open_fuzzy_search_in_files_and_focus();
                    None
                }
                MainViewMsg::CloseHover => {
                    if let Some(HoverItem::ContextMain { anchor, widget, old_focus }) = self.hover.take() {
                        self.set_focused(old_focus);
                    } else {
                        self.set_focus_to_default();
                    };

                    None
                }
                MainViewMsg::OpenChooseDisplay => {
                    self.open_fuzzy_buffer_list_and_focus();
                    None
                }
                MainViewMsg::OpenNewFile => {
                    self.open_empty_editor_and_focus();
                    None
                }
                MainViewMsg::FocusOnDisplay { display_idx: pos } => {
                    self.hover = None;

                    if *pos >= self.displays.len() {
                        error!(
                            "received FuzzyBufferHit for an index {} and len is {}, ignoring",
                            pos,
                            self.displays.len()
                        );
                    } else {
                        self.display_idx = *pos;
                        self.set_focus_to_default();
                    }

                    None
                }
                MainViewMsg::FindReferences { ref mut promise_op } => {
                    if let Some(promise) = promise_op.take() {
                        match self.create_new_display_for_code_results(Box::new(promise)) {
                            Ok(idx) => {
                                self.display_idx = idx;
                                self.set_focus_to_default();
                            }
                            Err(_) => {
                                error!("failed find references");
                            }
                        }
                    } else {
                        warn!("find reference with empty promise")
                    }
                    None
                }
                MainViewMsg::OpenFile { file, position_op } => {
                    self.open_document_and_focus(file.clone(), position_op.clone());
                    None
                }
                MainViewMsg::OpenFileBySpath { spath } => {
                    self.hover = None;
                    self.open_file_with_path_and_focus(spath.clone());
                    None
                }
                MainViewMsg::GoToDefinition { promise_op } => {
                    if let Some(promise) = promise_op.take() {
                        match self.create_new_display_for_code_results(Box::new(promise)) {
                            Ok(idx) => {
                                self.display_idx = idx;
                                self.set_focus_to_default();
                            }
                            Err(_) => {
                                error!("failed to go to definition ");
                            }
                        }
                    } else {
                        warn!("find reference with empty promise")
                    }
                    None
                }
                MainViewMsg::OpenFindInFiles => {
                    self.open_find_in_files();
                    None
                }
                MainViewMsg::FindInFilesQuery {
                    root_dir,
                    query,
                    filter_op,
                } => {
                    self.handle_open_find_in_files(root_dir.clone(), query.clone(), filter_op.take());
                    None
                }
                MainViewMsg::OpenContextMenu => {
                    self.open_main_context_menu_and_focus();
                    None
                }
                MainViewMsg::ContextMenuHit { msg, depth } => {
                    if let Some(HoverItem::ContextMain { anchor, widget, old_focus }) = self.hover.take() {
                        self.set_focused(old_focus);
                    }

                    if let Some(msg) = msg.take() {
                        self.context_menu_process_hit(msg, *depth)
                    } else {
                        None
                    }
                }
                MainViewMsg::PruneUnchangedBuffers => {
                    self.do_prune_unchanged_buffers();
                    None
                }

                MainViewMsg::QuitGladius => GladiusMsg::Quit.someboxed(),
                _ => {
                    warn!("unprocessed event {:?}", main_view_msg);
                    None
                }
            };
        };

        warn!("expecetd MainViewMsg got {:?}", msg);
        None
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.complex_get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.complex_get_focused_mut()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.complex_render(theme, focused, output)
    }

    fn get_status_description(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed("gladius"))
    }

    fn get_widget_actions(&self) -> Option<ContextBarItem> {
        let config = self.providers.config();

        Some(ContextBarItem::new_internal_node(
            Cow::Borrowed("gladius"),
            vec![
                ContextBarItem::new_leaf_node(
                    Cow::Borrowed("quit"),
                    || MainViewMsg::QuitGladius.boxed(),
                    Some(config.keyboard_config.global.close),
                ),
                ContextBarItem::new_leaf_node(
                    Cow::Borrowed("open display list"),
                    || MainViewMsg::OpenChooseDisplay.boxed(),
                    Some(config.keyboard_config.global.browse_buffers),
                ),
                ContextBarItem::new_leaf_node(
                    Cow::Borrowed("open new buffer"),
                    || MainViewMsg::OpenNewFile.boxed(),
                    Some(config.keyboard_config.global.new_buffer),
                ),
                ContextBarItem::new_leaf_node(
                    Cow::Borrowed("find in files"),
                    || MainViewMsg::OpenFindInFiles.boxed(),
                    Some(config.keyboard_config.global.find_in_files),
                ),
                ContextBarItem::new_leaf_node(
                    Cow::Borrowed("prune unchanged buffers"),
                    || MainViewMsg::PruneUnchangedBuffers.boxed(),
                    None,
                ),
            ],
        ))
    }

    fn kite(&self) -> XY {
        self.complex_kite()
    }
}

impl ComplexWidget for MainView {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let left_column = LeafLayout::new(subwidget!(Self.tree_widget)).boxed();
        let right_column = LeafLayout::new(self.get_curr_display_ptr()).boxed();

        let bottom_bar = LeafLayout::new(subwidget!(Self.status_bar));

        let bg_layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0), left_column)
            .with(SplitRule::Proportional(5.0), right_column);

        let bg_layout = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0), bg_layout.boxed())
            .with(SplitRule::Fixed(1), bottom_bar.boxed());

        let res = if self.hover.is_some() {
            let subwidget = self.get_hover_subwidget().unwrap();
            let leaf = LeafLayout::new(subwidget).boxed();

            let layout = match self.hover.as_ref().unwrap() {
                HoverItem::ContextMain { anchor, .. } => {
                    let anchor_clone = *anchor;
                    HoverLayout::new(
                        bg_layout.boxed(),
                        leaf,
                        Box::new(move |screenspace: Screenspace| util::get_rect_for_context_menu(screenspace.output_size(), anchor_clone)),
                        true,
                    )
                }
                _ => HoverLayout::new(bg_layout.boxed(), leaf, Box::new(Self::get_hover_rect), true),
            };

            layout.boxed()
        } else {
            bg_layout.boxed()
        };

        res
    }

    fn get_default_focused(&self) -> SubwidgetPointer<MainView> {
        self.get_curr_display_ptr()
    }

    fn set_display_state(&mut self, display_state: DisplayState<MainView>) {
        self.display_state = Some(display_state);
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<MainView>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for DocumentIdentifier {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            buffer_id: arbitrary::Arbitrary::arbitrary(u)?,
            file_path: None,
        })
    }
}
