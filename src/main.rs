#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;


use std::io::stdout;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use crossbeam_channel::select;
use filesystem::OsFileSystem;
use log::{debug, error, warn};
use termion::raw::IntoRawMode;

use crate::experiments::tree_sitter_wrapper::{LanguageSet, TreeSitterWrapper};
use crate::io::crossterm_input::CrosstermInput;
use crate::io::crossterm_output::CrosstermOutput;
use crate::io::filesystem_tree::filesystem_front::FsfRef;
use crate::io::filesystem_tree::local_filesystem_front::LocalFilesystem;
use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::theme::Theme;
use crate::primitives::xy::ZERO;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;
use crate::widgets::main_view::main_view::MainView;

mod experiments;
mod io;
mod layout;
mod primitives;
mod view;
mod widget;
mod text;
mod widgets;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity,
}


fn main() {
    let args = Args::parse();
    env_logger::builder()
        .filter_level(args.verbosity.log_level_filter())
        .init();

    let tree_sitter = Rc::new(TreeSitterWrapper::new(LanguageSet::full()));


    let stdout = stdout();
    let stdout = stdout.lock().into_raw_mode().unwrap();
    // let stdin = stdin();

    // write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    // stdout.flush().unwrap();

    let input = CrosstermInput::new();
    let mut output = CrosstermOutput::new(stdout);

    if output.size_constraint().hint().size == ZERO {
        //TODO
        return;
    }

    // let input = TermionInput::new(stdin);
    // let mut output = TermionOutput::new(stdout);

    // let mut main_view = TwoButtonEdit::new();

    // let stupid_tree = get_stupid_tree();
    // let mut main_view = TreeViewWidget::new(Box::new(stupid_tree));

    // let mut mock_list = get_mock_file_list();
    // let mut main_view = ListWidget::<MockFile>::new().with_items(mock_list)
    //     .with_selection();

    // let mut main_view = TextEditorWidget::new();

    // let fsp = LocalFilesystemProvider::new(PathBuf::from("/home/andrzej"));
    // let boxed = Box::new(fsp);
    // let mut main_view = SaveFileDialogWidget::new(boxed);

    let fsf: FsfRef = Rc::new(Box::new(LocalFilesystem::new(PathBuf::from("/home/andrzej/r"))));

    let mut main_view = MainView::new(tree_sitter, fsf.clone())
        .with_empty_editor();

    // let trash = Rc::new("trash".to_string());

    // let mut main_view = SaveFileDialogWidget::new(fsf.clone()).with_something_to_save(Box::new(trash));

    let theme = Theme::load_from_file(OsFileSystem::new(), &PathBuf::from("./themes/default.ron")).unwrap(); // TODO

    // returns (consumed, message_to_parent)
    fn recursive_treat_views(
        view: &mut dyn Widget,
        ie: InputEvent,
    ) -> (bool, Option<Box<dyn AnyMsg>>) {
        let my_desc = format!("{:?}", &view).clone();

        let focused_child_op = view.get_focused_mut();
        let child_desc = format!("{:?}", &focused_child_op);

        warn!("rtv0 {:?}: event {:?}, active_child: {:?}", my_desc, ie, child_desc);

        // first, dig as deep as possible.
        let (child_have_consumed, message_from_child_op) = match focused_child_op {
            Some(focused_child) => recursive_treat_views(focused_child, ie),
            None => (false, None)
        };
        warn!("rtv1 {:?}: event {:?}, active_child: {:?}, child_consumed: {}, message_from_child: {:?}",
            my_desc, ie, child_desc, child_have_consumed, &message_from_child_op);

        if child_have_consumed {
            return match message_from_child_op {
                None => (true, None),
                Some(message_from_child) => {
                    let msg_from_child_text = format!("{:?}", &message_from_child);
                    let my_message_to_parent = view.update(message_from_child);
                    debug!("rtv3 {:?}: message_from_child: {:?} sent to me, responding {:?} to parent",
                        my_desc, msg_from_child_text, &my_message_to_parent);
                    (true, my_message_to_parent)
                }
            };
        };

        // Either child did not consume (unwinding), or we're on the bottom of path.
        // We're here to consume the Input.
        match view.on_input(ie) {
            None => {
                debug!("rtv4 {:?}: did not consume {:?} either.", my_desc, ie);
                // we did not see this input as useful, unfolding the recursion:
                // no consume, no message.
                (false, None)
            }
            Some(internal_message) => {
                debug!("rtv5 {:?}: consumed {:?} and am pushing {:?} to myself", my_desc, ie, internal_message);
                let message_to_parent = view.update(internal_message);
                debug!("rtv6 {:?}: send {:?} to parent", my_desc, message_to_parent);
                // (message_to_parent.is_some(), message_to_parent)
                (true, message_to_parent)
            }
        }
    }

    'main:
    loop {
        output.clear();
        main_view.layout(output.size_constraint());
        main_view.render(&theme, true, &mut output);
        output.end_frame();

        select! {
            recv(input.source()) -> msg => {
                debug!("processing input: {:?}", msg);
                match msg {
                    Ok(mut ie) => {
                        debug!("{:?}", ie);
                        match ie {
                            InputEvent::KeyInput(key) if key.as_focus_update().is_some() => {
                                ie = InputEvent::FocusUpdate(key.as_focus_update().unwrap());
                            },
                            InputEvent::KeyInput(key) if key.keycode == Keycode::Char('q') && key.modifiers.CTRL => {
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
                debug!("processing tick: {:?}", msg);
                msg.map(|_| fsf.tick()).unwrap_or_else(|e| {
                    error!("failed receiving fsf_tick: {}", e);
                });
            }
        }
    }
}
