#![allow(dead_code)]
#![allow(unreachable_patterns)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate matches;

use std::io::stdout;
use std::path::PathBuf;
use std::rc::Rc;
use clap::Parser;

use crossbeam_channel::select;
use log::{debug, error};
use termion::raw::IntoRawMode;

use crate::io::crossterm_input::CrosstermInput;
use crate::io::crossterm_output::CrosstermOutput;
use fs::fsfref::FsfRef;
use fs::local_filesystem_front::LocalFilesystem;
use crate::experiments::clipboard::get_me_some_clipboard;
use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use config::theme::Theme;
use crate::config::config::{Config, ConfigRef};
use crate::primitives::xy::ZERO;
use crate::tsw::language_set::LanguageSet;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;
use crate::widgets::main_view::main_view::MainView;
use crate::gladius::load_config::load_config;
use crate::gladius::logger_setup::logger_setup;

mod experiments;
mod io;
mod layout;
mod primitives;
mod widget;
mod text;
mod widgets;
mod tsw;
mod fs;
mod config;
mod gladius;

fn main() {
    let args = gladius::args::Args::parse();
    logger_setup(args.verbosity.log_level_filter());

    let config_ref = load_config(args.reconfigure);
    let theme = Theme::load_or_create_default(&config_ref.config_dir).unwrap();

    let (start_dir, files) = args.paths();

    let fsf: FsfRef = LocalFilesystem::new(start_dir);

    let clipboard = get_me_some_clipboard();
    let tree_sitter = Rc::new(TreeSitterWrapper::new(LanguageSet::full()));

    let stdout = stdout();
    let stdout = stdout.lock().into_raw_mode().unwrap();

    let input = CrosstermInput::new();
    let mut output = CrosstermOutput::new(stdout);

    if output.size_constraint().visible_hint().size == ZERO {
        //TODO
        return;
    }

    let mut main_view = MainView::new(config_ref.clone(), tree_sitter, fsf.clone(), clipboard);

    // returns (consumed, message_to_parent)
    fn recursive_treat_views(
        view: &mut dyn Widget,
        ie: InputEvent,
    ) -> (bool, Option<Box<dyn AnyMsg>>) {
        let my_desc = format!("{:?}", &view).clone();

        let focused_child_op = view.get_focused_mut();
        let child_desc = format!("{:?}", &focused_child_op);

        debug!(target: "recursive_treat_views", "{:?}: event {:?}, active_child: {:?}", my_desc, ie, child_desc);

        // first, dig as deep as possible.
        let (child_have_consumed, message_from_child_op) = match focused_child_op {
            Some(focused_child) => recursive_treat_views(focused_child, ie),
            None => (false, None)
        };
        debug!(target: "recursive_treat_views", "{:?}: event {:?}, active_child: {:?}, child_consumed: {}, message_from_child: {:?}",
            my_desc, ie, child_desc, child_have_consumed, &message_from_child_op);

        if child_have_consumed {
            return match message_from_child_op {
                None => (true, None),
                Some(message_from_child) => {
                    let msg_from_child_text = format!("{:?}", &message_from_child);
                    let my_message_to_parent = view.update(message_from_child);
                    debug!(target: "recursive_treat_views", "{:?}: message_from_child: {:?} sent to me, responding {:?} to parent",
                        my_desc, msg_from_child_text, &my_message_to_parent);
                    (true, my_message_to_parent)
                }
            };
        };

        // Either child did not consume (unwinding), or we're on the bottom of path.
        // We're here to consume the Input.
        match view.on_input(ie) {
            None => {
                debug!(target: "recursive_treat_views", "{:?}: did not consume {:?} either.", my_desc, ie);
                // we did not see this input as useful, unfolding the recursion:
                // no consume, no message.
                (false, None)
            }
            Some(internal_message) => {
                debug!(target: "recursive_treat_views", "{:?}: consumed {:?} and am pushing {:?} to myself", my_desc, ie, internal_message);
                let message_to_parent = view.update(internal_message);
                debug!(target: "recursive_treat_views", "{:?}: send {:?} to parent", my_desc, message_to_parent);
                // (message_to_parent.is_some(), message_to_parent)
                (true, message_to_parent)
            }
        }
    }

    'main:
    loop {
        match output.clear() {
            Ok(_) => {}
            Err(e) => {
                error!("failed to clear output: {}", e);
                break;
            }
        }
        main_view.layout(output.size_constraint());
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
                            InputEvent::KeyInput(key) if key == config_ref.keyboard_config.global.everything_bar => {
                                ie = InputEvent::EverythingBarTrigger;
                            }
                            // TODO move to message, to handle signals in the same way?
                            InputEvent::KeyInput(key) if key == config_ref.keyboard_config.global.close => {
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
            recv(fsf.tick_recv()) -> msg => {
                // debug!("processing tick: {:?}", msg);
                msg.map(|_| fsf.tick()).unwrap_or_else(|e| {
                    error!("failed receiving fsf_tick: {}", e);
                });
            }
        }
    }
}
