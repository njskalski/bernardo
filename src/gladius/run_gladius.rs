use std::path::PathBuf;
use std::rc::Rc;

use crossbeam_channel::select;
use log::{debug, error};

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::gladius::paradigm::recursive_treat_views;
use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::output::FinalOutput;
use crate::tsw::language_set::LanguageSet;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::inspector::{inspect_workspace, InspectError};
use crate::w7e::workspace::{LoadError, ScopeLoadErrors, Workspace};
use crate::w7e::workspace::WORKSPACE_FILE_NAME;
use crate::widget::widget::Widget;
use crate::widgets::main_view::main_view::MainView;

pub fn run_gladius<
    I: Input,
    O: FinalOutput>(
    fsf: FsfRef,
    config: ConfigRef,
    clipboard: ClipboardRef,
    input: I,
    mut output: O,
    files: Vec<PathBuf>,
    theme: &Theme,
) {
    let tree_sitter = Rc::new(TreeSitterWrapper::new(LanguageSet::full()));

    // Loading / Building workspace file
    let workspace_dir = fsf.root();
    let (workspace_op, _scope_errors): (Option<Workspace>, ScopeLoadErrors) = match Workspace::try_load(workspace_dir.clone()) {
        Ok(res) => (Some(res.0), res.1),
        Err(e) => {
            match e {
                LoadError::WorkspaceFileNotFound => {
                    (None, ScopeLoadErrors::default())
                }
                LoadError::ReadError(e) => {
                    error!("failed reading workspace file at {}, because:\n{}\nterminating. To continue, rename/remove {} in that folder.",
                        fsf.root(), e, WORKSPACE_FILE_NAME);

                    return;
                }
            }
        }
    };

    // TODO add option to NOT *save* workspace after creation?
    let mut workspace: Workspace = match workspace_op {
        Some(w) => w,
        None => {
            // Attempting to create a reasonable workspace
            match inspect_workspace(&workspace_dir) {
                Err(e) => {
                    match &e {
                        InspectError::NotAFolder => {
                            error!("failed inspecting workspace at {:?}, because it doesn't seem to be a folder.",
                            workspace_dir.absolute_path());
                            // This should never happen, so I terminate program.
                            return;
                        }
                        _ => {
                            error!("failed inspecting workspace at {:?}, because:\n{}", workspace_dir.absolute_path(), e);
                            // I decided inspection should not have non-fatal errors, just worst case scenario being "no scopes".
                            return;
                        }
                    }
                }
                Ok(scopes) => {
                    debug!("creating new workspace at {:?} with {} scopes", workspace_dir.absolute_path(), scopes.len());
                    let workspace = Workspace::new(workspace_dir, scopes);
                    match workspace.save() {
                        Ok(_) => {
                            debug!("saved successfully");
                        }
                        Err(e) => {
                            error!("failed writing workspace file: {:?}", e);
                        }
                    }
                    workspace
                }
            }
        }
    };

    // At this point it is guaranteed that we have a Workspace present, though it might be not saved!

    // Initializing handlers
    let (nav_comp_group_ref, scope_errors) = workspace.initialize_handlers(&config);
    if !scope_errors.is_empty() {
        debug!("{} handlers failed to load, details : {:?}", scope_errors.len(), scope_errors);
    }

    let mut main_view = MainView::new(
        config.clone(),
        tree_sitter,
        fsf.clone(),
        clipboard,
        nav_comp_group_ref.clone(),
    );
    for f in files.iter() {
        if !fsf.descendant_checked(f).map(|ff| main_view.open_file(ff)).unwrap_or(false) {
            error!("failed opening file {:?}", f);
        }
    }

    // Genesis
    'main:
    loop {
        match output.clear() {
            Ok(_) => {}
            Err(e) => {
                error!("failed to clear output: {}", e);
                break;
            }
        }
        main_view.update_and_layout(output.size_constraint());
        main_view.render(&theme, true, &mut output);
        match output.end_frame() {
            Ok(_) => {}
            Err(e) => {
                error!("failed to end frame: {}", e);
                break;
            }
        }

        select! {
            recv(input.source()) -> msg => {
                // debug!("processing input: {:?}", msg);
                match msg {
                    Ok(mut ie) => {
                        // debug!("msg ie {:?}", ie);
                        match ie {
                            InputEvent::KeyInput(key) if key.as_focus_update().is_some() && key.modifiers.alt => {
                                ie = InputEvent::FocusUpdate(key.as_focus_update().unwrap());
                            },
                            InputEvent::KeyInput(key) if key == config.keyboard_config.global.everything_bar => {
                                ie = InputEvent::EverythingBarTrigger;
                            }
                            // TODO move to message, to handle signals in the same way?
                            InputEvent::KeyInput(key) if key == config.keyboard_config.global.close => {
                                break 'main;
                            }
                            _ => {}
                        }

                        recursive_treat_views(&mut main_view, ie);
                    },
                    Err(e) => {
                        error!("failed receiving input: {}", e);
                    }
                };
            }

            recv(nav_comp_group_ref.recvr()) -> tick => {
                match tick {
                    _ => {
                        // warn!("unhandled tick : {:?}", tick)
                    }
                }
            }
        }
    }
}