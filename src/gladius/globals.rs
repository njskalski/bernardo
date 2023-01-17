/*
This is a simplified "dependency injection" struct, because I just have too much items in constructors
of key components like EditorView or CodeResultsView
 */

use std::rc::Rc;
use std::sync::Arc;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::gladius::navcomp_loader::NavCompLoader;
use crate::gladius::sidechannel::x::SideChannel;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;

pub struct Globals {
    config: ConfigRef,
    fsf: FsfRef,
    clipboard: ClipboardRef,
    theme: Theme,
    tree_sitter: Arc<TreeSitterWrapper>,

    // side_channel: Option<SideChannel>,
    navcomp_loader: Arc<Box<dyn NavCompLoader>>,
}

pub type GlobalsRef = Arc<Globals>;

impl Globals {
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
        Globals {
            config,
            fsf,
            clipboard,
            theme,
            tree_sitter,
            // side_channel: None,
            navcomp_loader,
        }
    }

    // pub fn with_side_channel(self, side_channel: SideChannel) -> Self {
    //     Self {
    //         side_channel: Some(side_channel),
    //         ..self
    //     }
    // }

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