/*
This is a simplified "dependency injection" struct, because I just have too much items in constructors
of key components like EditorView or CodeResultsView
 */
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::buffer_register::{BufferRegister, BufferRegisterRef};
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::gladius::navcomp_loader::NavCompLoader;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::navcomp_group::NavCompGroup;
use crate::widgets::editor_widget::label::labels_provider::LabelsProviderRef;

// do not share via Arc, we want to be able to "overload" providers in tests or exotic cases
#[derive(Clone)]
pub struct Providers {
    config: ConfigRef,
    fsf: FsfRef,
    clipboard: ClipboardRef,
    theme: Theme,
    tree_sitter: Arc<TreeSitterWrapper>,

    navcomp_loader: Arc<Box<dyn NavCompLoader>>,
    navcomp_group: Arc<RwLock<NavCompGroup>>,

    buffer_register: BufferRegisterRef,

    todo_labels_providers: Vec<LabelsProviderRef>,
}

impl Debug for Providers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[providers]")
    }
}

impl Providers {
    pub fn new(
        config: ConfigRef,
        fsf: FsfRef,
        clipboard: ClipboardRef,
        theme: Theme,
        tree_sitter: Arc<TreeSitterWrapper>,
        /*
        I am not sure this shouldn't be part of workspace, but for now it carries no
        information, just implementation so I'll keep it here.
         */
        navcomp_loader: Arc<Box<dyn NavCompLoader>>,
        todo_labels_providers: Vec<LabelsProviderRef>,
    ) -> Self {
        Providers {
            config,
            fsf,
            clipboard,
            theme,
            tree_sitter,
            navcomp_loader,
            navcomp_group: Arc::new(RwLock::new(NavCompGroup::new())),
            buffer_register: Arc::new(RwLock::new(BufferRegister::new())),
            todo_labels_providers,
        }
    }

    pub fn fsf(&self) -> &FsfRef {
        &self.fsf
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn is_recording(&self) -> bool {
        false
    }

    pub fn config(&self) -> &ConfigRef {
        &self.config
    }

    pub fn navcomp_loader(&self) -> &dyn NavCompLoader {
        self.navcomp_loader.as_ref().as_ref()
    }

    pub fn tree_sitter(&self) -> &Arc<TreeSitterWrapper> {
        &self.tree_sitter
    }

    pub fn clipboard(&self) -> &ClipboardRef {
        &self.clipboard
    }

    pub fn buffer_register(&self) -> &BufferRegisterRef {
        &self.buffer_register
    }

    pub fn navcomp_group(&self) -> &Arc<RwLock<NavCompGroup>> {
        &self.navcomp_group
    }

    pub fn todo_label_providers(&self) -> impl Iterator<Item = &LabelsProviderRef> {
        self.todo_labels_providers.iter()
    }
}
