use log::LevelFilter;
use std::io::{stdin, stdout, Read, Write};
use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, color, cursor, style};
use crate::io::termion_output::TermionOutput;
use crate::io::termion_input::TermionInput;
use crate::io::output::Output;
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::experiments::two_button_edit::{TwoButtonEdit, TBEMsg};
use crate::widget::widget::Widget;

use log::debug;

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

    let mut view = TwoButtonEdit::new();

    loop {
        output.clear();

        match input.source().recv() {
            Ok(ie) => {
                debug!("{:?}", ie);
                match ie {
                    InputEvent::Tick => {}
                    InputEvent::KeyInput(key) => match key {
                        Key::CtrlLetter('q') => break,
                        _ => {
                            view.render(true, &mut output);
                        }
                    },
                    _ => {}
                }
            }
            Err(e) => {
                debug!("Err {:?}", e);
            }
        }

        output.end_frame();
    }
}
