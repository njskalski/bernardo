/*
This is a simplified "dependency injection" struct, because I just have too much items in constructors
of key components like EditorView or CodeResultsView
 */


use std::sync::Arc;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::gladius::navcomp_loader::NavCompLoader;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;

// do not share via Arc, we want to be able to "overload" providers in tests or exotic cases
#[derive(Clone)]
pub struct Providers {
    config: ConfigRef,
    fsf: FsfRef,
    clipboard: ClipboardRef,
    theme: Theme,
    tree_sitter: Arc<TreeSitterWrapper>,

    // side_channel: Option<SideChannel>,
    navcomp_loader: Arc<Box<dyn NavCompLoader>>,
}

impl Providers {
    pub fn new(config: ConfigRef,
               fsf: FsfRef,
               clipboard: ClipboardRef,
               theme: Theme,
               tree_sitter: Arc<TreeSitterWrapper>,
               /*
               I am not sure this shouldn't be part of workspace, but for now it carries no
               information, just implementation so I'll keep it here.
                */
               navcomp_loader: Arc<Box<dyn NavCompLoader>>,
    ) -> Self {
        Providers {
            config,
            fsf,
            clipboard,
            theme,
            tree_sitter,
            navcomp_loader,
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
}