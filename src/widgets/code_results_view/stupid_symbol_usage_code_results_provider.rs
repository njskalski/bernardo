use std::iter;
use std::sync::Arc;

use log::{debug, error};

use crate::gladius::providers::Providers;
use crate::io::loading_state::LoadingState;
use crate::primitives::printable::Printable;
use crate::primitives::symbol_usage::SymbolUsage;
use crate::promise::promise::PromiseState;
use crate::text::text_buffer::TextBuffer;
use crate::unpack_unit_e;
use crate::w7e::navcomp_provider::{StupidSymbolUsage, SymbolUsagesPromise};
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;

#[derive(Debug)]
pub struct StupidSymbolUsageCodeResultsProvider {
    providers: Providers,
    desc: Arc<String>,
    promise: Option<SymbolUsagesPromise>,
    resolved_symbols: Vec<SymbolUsage>,
}

impl StupidSymbolUsageCodeResultsProvider {
    pub fn new(providers: Providers, desc: String, promise: SymbolUsagesPromise) -> Self {
        StupidSymbolUsageCodeResultsProvider {
            providers,
            desc: Arc::new(desc),
            promise: Some(promise),
            resolved_symbols: vec![],
        }
    }

    fn add_stupid_symbols(&mut self, stupid_symbols: &Vec<StupidSymbolUsage>) {
        let mut buffer_register_lock = unpack_unit_e!(
            self.providers.buffer_register().try_write().ok(),
            "failed to acquire buffer register",
        );

        let root_path_buf = self.providers.fsf().root_path_buf().to_string_lossy().to_string() + "/";

        for ss in stupid_symbols.iter() {
            let no_prefix = match ss.path.strip_prefix("file://") {
                None => {
                    error!("failed stripping prefix file:// from {}", &ss.path);
                    continue;
                }
                Some(np) => np,
            };

            let in_workspace = match no_prefix.strip_prefix(&root_path_buf) {
                None => {
                    error!("failed stripping prefix root_path from {}", &no_prefix);
                    continue;
                }
                Some(iw) => iw,
            };

            let spath = match self.providers.fsf().descendant_checked(&in_workspace) {
                None => {
                    error!("failed to get spath from {}", &in_workspace);
                    continue;
                }
                Some(s) => s,
            };

            let open_result = buffer_register_lock.open_file(&self.providers, &spath);

            let buffer_state_ref = match open_result.buffer_shared_ref {
                Err(e) => {
                    error!("failed to load buffer {} because {}", spath, e);
                    continue;
                }
                Ok(bsr) => bsr,
            };

            let lock = match buffer_state_ref.lock() {
                None => {
                    error!("failed locking buffer {}", spath);
                    continue;
                }
                Some(lock) => lock,
            };

            let cursor = if ss.stupid_range.0 == ss.stupid_range.1 {
                lock.stupid_cursor_to_cursor(ss.stupid_range.0, None)
            } else {
                lock.stupid_cursor_to_cursor(ss.stupid_range.0, Some(ss.stupid_range.1))
            };

            let cursor = match cursor {
                None => {
                    error!("failed cursor conversion");
                    continue;
                }
                Some(c) => c,
            };

            let su = SymbolUsage {
                path: spath,
                range: cursor,
            };

            self.resolved_symbols.push(su);
        }
    }
}

impl CodeResultsProvider for StupidSymbolUsageCodeResultsProvider {
    fn description(&self) -> Box<dyn Printable> {
        Box::new(self.desc.clone())
    }

    fn poll(&mut self) {
        if let Some(promise) = self.promise.as_mut() {
            let update_result = promise.update();
            if update_result.state.is_broken() {
                error!("broken promise of code results");
            }

            if update_result.state.is_resolved() {
                if let Some(items) = self.promise.as_ref().unwrap().read() {
                    let items_len = items.len();
                    // TODO unnecessary clone
                    self.add_stupid_symbols(&items.clone());
                    debug!("added {} of {} symbols", self.resolved_symbols.len(), items_len);
                }
                self.promise = None;
            }

            debug!("ticking result: {:?}", update_result);
        }
    }

    fn loading_state(&self) -> LoadingState {
        if let Some(promise) = self.promise.as_ref() {
            match promise.state() {
                PromiseState::Unresolved => LoadingState::InProgress,
                PromiseState::Ready => LoadingState::Complete,
                PromiseState::Broken => LoadingState::Error,
            }
        } else {
            // TODO eaten an error
            LoadingState::Complete
        }
    }

    // TODO this entire method is a stub. It should not copy, it should stream and stuff.
    fn items(&self) -> Box<dyn Iterator<Item=&SymbolUsage> + '_> {
        if self.promise.is_some() {
            Box::new(iter::empty()) as Box<dyn Iterator<Item=&SymbolUsage>>
        } else {
            Box::new(self.resolved_symbols.iter())
        }
    }
}
