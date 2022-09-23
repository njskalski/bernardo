use std::path::PathBuf;

use clap::Parser;
use crossbeam_channel::select;
use crossterm::terminal;
use log::error;

use bernardo::config::theme::Theme;
use bernardo::gladius::paradigm::recursive_treat_views;
use bernardo::io::buffer::Buffer;
use bernardo::io::buffer_output::BufferOutput;
use bernardo::io::cell::Cell;
use bernardo::io::crossterm_input::CrosstermInput;
use bernardo::io::crossterm_output::CrosstermOutput;
use bernardo::io::input::Input;
use bernardo::io::input_event::InputEvent;
use bernardo::io::keys::Keycode;
use bernardo::io::output::{FinalOutput, Output};
use bernardo::widget::widget::Widget;

use crate::reader_main_widget::ReaderMainWidget;

mod reader_main_widget;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    pub file: String,
}

fn main() {
    env_logger::builder().init();

    let args = Args::parse();

    let dump = match BufferOutput::from_file(&args.file) {
        Ok(d) => d,
        Err(_) => {
            std::process::exit(1);
            return;
        }
    };

    // Initializing Bernardo TUI
    terminal::enable_raw_mode().expect("failed entering raw mode");
    let input = CrosstermInput::new();
    let stdout = std::io::stdout();
    let mut output = CrosstermOutput::new(stdout);

    let theme = Theme::default();
    let mut main_view = ReaderMainWidget::new(dump);

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
                            // InputEvent::KeyInput(key) if key.as_focus_update().is_some() && key.modifiers.alt => {
                            //     ie = InputEvent::FocusUpdate(key.as_focus_update().unwrap());
                            // },
                            // TODO move to message, to handle signals in the same way?
                            InputEvent::KeyInput(key) if key.keycode == Keycode::Esc => {
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
        }
    }
}