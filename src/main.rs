use log::LevelFilter;
use std::io::{stdin, stdout, Read, Write};
use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, color, cursor, style};
use crate::io::termion_output::TermionOutput;
use crate::io::termion_input::TermionInput;
use crate::io::output::Output;
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::experiments::two_button_edit::{TwoButtonEdit, TwoButtonEditMsg};
use crate::widget::widget::Widget;
use crate::primitives::xy::XY;

use log::debug;
use crate::widget::any_msg::AnyMsg;
use crate::primitives::xy;

mod io;
mod primitives;
mod view;
mod widget;
mod experiments;
mod layout;


fn main() {
    env_logger::builder()
        .filter(None, LevelFilter::Debug)
        .init();

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();

    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    let input = TermionInput::new(stdin);
    let mut output = TermionOutput::new(stdout);

    let mut main_view = TwoButtonEdit::new();

    fn recursive_treat_views(view : &mut dyn Widget, ie : InputEvent) -> (bool, Option<Box<dyn AnyMsg>>) {
        let my_id = view.id();
        let active_child_id = view.get_focused().id();

        // this is my turn
        let (child_have_consumed, message_from_child_op) = if my_id != active_child_id {
            recursive_treat_views(view.get_focused_mut(), ie.clone())
        } else {
            (false, None)
        };

        if child_have_consumed {
            match message_from_child_op {
                None => {
                    return (true, None)
                }
                Some(message_from_child) => {
                    let my_message_to_parent = view.update(message_from_child);
                    return (true, my_message_to_parent)
                }
            }
        };

        // Either child did not consume, or we're on the bottom of path.
        // We're here to consume the Input.
        match view.on_input(ie) {
            None => {
                // we did not see this input as useful, unfolding the recursion:
                // no consume, no message.
                (false, None)
            }
            Some(internal_message) => {
                let message_to_parent = view.update(internal_message);
                (true, message_to_parent)
            }
        }
    };

    loop {
        // at this point we have path and

        match input.source().recv() {
            Ok(ie) => {
                debug!("{:?}", ie);
                // early exit
                match ie {
                    InputEvent::KeyInput(Key::CtrlLetter(q)) => break,
                    _ => {}
                }
                recursive_treat_views(&mut main_view, ie);
            }
            Err(e) => {
                debug!("Err {:?}", e);
            }
        }

        output.clear();
        main_view.render(true, xy::Zero, &mut output);
        output.end_frame();
    }
}
