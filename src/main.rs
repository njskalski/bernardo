use std::fmt::Debug;
use std::io::{Read, stdin, stdout, Write};

use log::{debug, warn};
use log::LevelFilter;
use termion::{async_stdin, clear, color, cursor, style};
use termion::raw::IntoRawMode;

use crate::experiments::save_file_dialog::SaveFileDialogWidget;
use crate::io::crossterm_input::CrosstermInput;
use crate::io::crossterm_output::CrosstermOutput;
use crate::io::input::Input;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::io::termion_input::TermionInput;
use crate::io::termion_output::TermionOutput;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::list_widget::ListWidget;
use crate::widget::mock_file_list::mock::{get_mock_file_list, MockFile};
use crate::widget::stupid_tree::get_stupid_tree;
use crate::widget::text_editor::TextEditorWidget;
use crate::widget::tree_view::TreeViewWidget;
use crate::widget::widget::Widget;

mod experiments;
mod io;
mod layout;
mod primitives;
mod view;
mod widget;
mod text;

fn main() {
    env_logger::builder()
        .filter(None, LevelFilter::Debug)
        .init();

    let stdout = stdout();
    // let mut stdout = stdout.lock().into_raw_mode().unwrap();
    // let stdin = stdin();

    // write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    // stdout.flush().unwrap();

    let input = CrosstermInput::new();
    let mut output = CrosstermOutput::new(stdout);

    // let input = TermionInput::new(stdin);
    // let mut output = TermionOutput::new(stdout);

    // let mut main_view = TwoButtonEdit::new();

    // let stupid_tree = get_stupid_tree();
    // let mut main_view = TreeViewWidget::new(Box::new(stupid_tree));

    // let mut mock_list = get_mock_file_list();
    // let mut main_view = ListWidget::<MockFile>::new().with_items(mock_list)
    //     .with_selection();

    // let mut main_view = TextEditorWidget::new();

    let mut main_view = SaveFileDialogWidget::new();

    // returns (consumed, message_to_parent)
    fn recursive_treat_views(
        view: &mut dyn Widget,
        ie: InputEvent,
    ) -> (bool, Option<Box<dyn AnyMsg>>) {
        let my_id = view.id();
        let active_child_id = view.get_focused().id();


        debug!("recursive_treat_views my_id {} aci {}", my_id, active_child_id);

        // first, dig as deep as possible.
        let (child_have_consumed, message_from_child_op) = if my_id != active_child_id {
            recursive_treat_views(view.get_focused_mut(), ie.clone())
        } else {
            (false, None)
        };

        if child_have_consumed {
            debug!("child {} consumed", active_child_id);

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

        debug!("child {} did not consume", active_child_id);

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
        main_view.layout(output.size());
        main_view.render(true, &mut output);
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
