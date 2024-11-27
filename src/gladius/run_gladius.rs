use std::fs;
use std::path::PathBuf;

use crossbeam_channel::select;
use log::{debug, error};

use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::msg::GladiusMsg;
use crate::gladius::providers::Providers;
use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::FinalOutput;
use crate::primitives::helpers::get_next_filename;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::inspector::{inspect_workspace, InspectError};
use crate::w7e::workspace::WORKSPACE_FILE_NAME;
use crate::w7e::workspace::{LoadError, ScopeLoadErrors, Workspace};
use crate::widget::widget::Widget;
use crate::widgets::main_view::main_view::MainView;

pub fn run_gladius<I: Input, O: FinalOutput>(providers: Providers, input: I, mut output: O, files: Vec<PathBuf>) {
    // Loading / Building workspace file
    let workspace_dir = providers.fsf().root();
    let (workspace_op, _scope_errors): (Option<Workspace>, ScopeLoadErrors) = match Workspace::try_load(workspace_dir.clone()) {
        Ok(res) => (Some(res.0), res.1),
        Err(e) => match e {
            LoadError::WorkspaceFileNotFound => (None, ScopeLoadErrors::default()),
            LoadError::ReadError(e) => {
                error!(
                    "failed reading workspace file at {}, because:\n{}\nterminating. To continue, rename/remove {} in that folder.",
                    workspace_dir, e, WORKSPACE_FILE_NAME
                );

                return;
            }
        },
    };

    // TODO add option to NOT *save* workspace after creation?
    let mut workspace: Workspace = match workspace_op {
        Some(w) => w,
        None => {
            // Attempting to create a reasonable workspace
            #[allow(unreachable_patterns)]
            match inspect_workspace(&workspace_dir) {
                Err(e) => {
                    match &e {
                        InspectError::NotAFolder => {
                            error!(
                                "failed inspecting workspace at {:?}, because it doesn't seem to be a folder.",
                                workspace_dir.absolute_path()
                            );
                            // This should never happen, so I terminate program.
                            return;
                        }
                        _ => {
                            error!(
                                "failed inspecting workspace at {:?}, because:\n{}",
                                workspace_dir.absolute_path(),
                                e
                            );
                            // I decided inspection should not have non-fatal errors, just worst case scenario being "no scopes".
                            return;
                        }
                    }
                }
                Ok(scopes) => {
                    debug!(
                        "creating new workspace at {:?} with {} scopes",
                        workspace_dir.absolute_path(),
                        scopes.len()
                    );
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
    let scope_errors: Vec<HandlerLoadError> = match workspace.initialize_handlers(providers.clone()) {
        Ok(maybe_errors) => maybe_errors,
        Err(_) => {
            error!("early, unrecoverable sync error");
            return;
        }
    };

    if !scope_errors.is_empty() {
        debug!("{} handlers failed to load, details : {:?}", scope_errors.len(), scope_errors);
    }

    let mut main_view = MainView::new(providers.clone());
    for f in files.iter() {
        if !providers
            .fsf()
            .descendant_checked(f)
            .map(|ff| main_view.open_file_with_path_and_focus(ff))
            .unwrap_or(false)
        {
            error!("failed opening file {:?}", f);
        }
    }

    let mut recorded_input: Vec<InputEvent> = Vec::new();

    let nav_comp_tick_receiver = providers.navcomp_group().try_read().map(|lock| lock.recvr().clone()).unwrap(); // TODO unwrap

    // Genesis
    'main: loop {
        debug!("new frame");
        match output.clear() {
            Ok(_) => {}
            Err(e) => {
                error!("failed to clear output: {}", e);
                break;
            }
        }
        main_view.prelayout();
        let output_size = output.size();

        main_view.layout(Screenspace::full_output(output_size));
        main_view.render(providers.theme(), true, &mut output);
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
                        if providers.is_recording() {
                            recorded_input.push(ie.clone());
                        }

                        match ie {
                            InputEvent::KeyInput(key) if key.as_focus_update().is_some() && key.modifiers.alt => {
                                ie = InputEvent::FocusUpdate(key.as_focus_update().unwrap());
                            },
                            InputEvent::KeyInput(key) if key == providers.config().keyboard_config.global.everything_bar => {
                                ie = InputEvent::EverythingBarTrigger;
                            }
                            _ => {}
                        }

                        let (_ , result_op) = main_view.act_on(ie);
                        if let Some(result) = result_op.as_ref() {
                            if let Some(gladius_msg) = result.as_msg::<GladiusMsg>() {
                                match gladius_msg {
                                    GladiusMsg::Quit => {
                                        break 'main;
                                    }
                                    GladiusMsg::Screenshot => {
                                        let buffer = output.get_front_buffer();
                                        screenshot(&buffer);
                                    }
                                }
                            } else {
                                error!("failed processing output msg {:?}", result);
                            }
                        }
                    },
                    Err(e) => {
                        error!("failed receiving input: {}", e);
                    }
                };
            }

            recv(nav_comp_tick_receiver) -> tick => {

                if providers.is_recording() {
                    recorded_input.push(InputEvent::Tick);
                }

                match tick {
                    _ => {
                        // warn!("unhandled tick : {:?}", tick)
                    }
                }
            }
        }
    }

    if providers.is_recording() {
        let bytes = match ron::to_string(&recorded_input) {
            Ok(b) => b,
            Err(e) => {
                error!("failed to serialzie recording: {:?}", e);
                return;
            }
        };

        let recordings_path = PathBuf::from("./input_recordings/");
        if let Err(e) = fs::create_dir_all(&recordings_path) {
            error!("failed to create dir for recordings: {:?}", e);
            return;
        }

        let filename = match get_next_filename(&recordings_path, "recording_", ".ron") {
            Some(f) => f,
            None => {
                error!("no filename for recording");
                return;
            }
        };

        match fs::write(filename, bytes) {
            Ok(_) => {}
            Err(e) => {
                error!("failed to write recordings: {:?}", e);
            }
        }
    }
}
