use log::LevelFilter;
use std::io::{stdin, stdout, Read, Write};
use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, color, cursor, style};
use crate::io::termion_output::TermionOutput;
use crate::io::termion_input::TermionInput;
use crate::io::output::Output;

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

    // loop {
    //     switchboard.borrow_mut().exhaust_ticks(None);
    //
    //     output
    //         .update_size()
    //         .map(|new_size| root_view.set_size(new_size));
    //
    //     output.clear();
    //
    //     root_view.render(&style_provider, FocusType::Hovered, &mut output);
    //
    //     output.end_frame();
    //
    //     match input.source().recv() {
    //         // Ok(ie) => {
    //         //     debug!("{:?}", ie);
    //         //     match ie {
    //         //         InputEvent::Tick => {}
    //         //         InputEvent::KeyInput(key) => match key {
    //         //             Key::CtrlLetter('q') => break,
    //         //             _ => {
    //         //                 root_view.consume_input(key);
    //         //             }
    //         //         },
    //         //         _ => {}
    //         //     }
    //         // }
    //         Err(e) => {
    //             debug!("Err {:?}", e);
    //         }
    //     }
    // }

}
