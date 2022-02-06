use std::env::args;
use std::io::stdout;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use log::debug;
use termion::raw::IntoRawMode;

use crate::experiments::color_theme::ColorTheme;
use crate::experiments::tree_sitter_wrapper::{LanguageSet, TreeSitterWrapper};
use crate::io::crossterm_input::CrosstermInput;
use crate::io::crossterm_output::CrosstermOutput;
use crate::io::filesystem_tree::local_filesystem_provider::LocalFilesystemProvider;
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

    let tree_sitter_wrapper = TreeSitterWrapper::new(LanguageSet::full());


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

    let mut main_view = MainView::new(PathBuf::from("/home/andrzej/r"))
        .with_empty_editor()
        .with_tree_sitter(Rc::new(tree_sitter_wrapper));

    let fs = filesystem::OsFileSystem::new();
    let color_theme = ColorTheme::load_from_file(fs, &PathBuf::from("./themes/default.ron")).unwrap(); // TODO

    let theme = Theme::default().with_color_theme(color_theme);

    // returns (consumed, message_to_parent)
    fn recursive_treat_views(
        view: &mut dyn Widget,
        ie: InputEvent,
    ) -> (bool, Option<Box<dyn AnyMsg>>) {
        let my_id = view.id();
        let focused_child_op = view.get_focused_mut();
        let active_child_id_op = focused_child_op.as_ref().map(|w| w.id());

        debug!("recursive_treat_views my_id {} aci {:?}", my_id, active_child_id_op);

        // first, dig as deep as possible.
        let (child_have_consumed, message_from_child_op) = match focused_child_op {
            Some(focused_child) => recursive_treat_views(focused_child, ie),
            None => (false, None)
        };

        if child_have_consumed {
            debug!("child {:?} consumed", active_child_id_op);

            return match message_from_child_op {
                None => (true, None),
                Some(message_from_child) => {
                    debug!("cs pushing {:?} to {}", message_from_child, view.typename());
                    let my_message_to_parent = view.update(message_from_child);
                    debug!("cs resp {:?}", my_message_to_parent);
                    (true, my_message_to_parent)
                }
            }
        };

        debug!("child {:?} did not consume", active_child_id_op);

        // Either child did not consume (unwinding), or we're on the bottom of path.
        // We're here to consume the Input.
        match view.on_input(ie) {
            None => {
                debug!("{} did not consume either.", my_id);
                // we did not see this input as useful, unfolding the recursion:
                // no consume, no message.
                (false, None)
            }
            Some(internal_message) => {
                debug!("uw pushing {:?} to {}", internal_message, view.typename());
                let message_to_parent = view.update(internal_message);
                debug!("uw resp {:?}", message_to_parent);
                (message_to_parent.is_some(), message_to_parent)
            }
        }
    }

    loop {
        output.clear();
        main_view.layout(output.size_constraint());
        main_view.render(&theme, true, &mut output);
        output.end_frame();

        match input.source().recv() {
            Ok(ie) => {
                debug!("{:?}", ie);
                // early exit
                match ie {
                    InputEvent::KeyInput(key) => {
                        match key.keycode {
                            Keycode::Char('q') if key.modifiers.CTRL => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                recursive_treat_views(&mut main_view, ie);
            }
            Err(e) => {
                debug!("Err {:?}", e);
            }
        }
    }
}
