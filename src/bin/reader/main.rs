use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use clap::Parser;
use crossbeam_channel::select;
use log::error;

use bernardo::app::App;
use bernardo::config::theme::Theme;
use bernardo::experiments::screenspace::Screenspace;
use bernardo::io::buffer_output::buffer_output::BufferOutput;
use bernardo::io::input::Input;
use bernardo::io::input_event::InputEvent;
use bernardo::io::keys::Keycode;
use bernardo::io::output::{FinalOutput, Output};
use bernardo::primitives::rect::Rect;
use bernardo::primitives::sized_xy::SizedXY;
use bernardo::widget::widget::Widget;

use crate::reader_main_widget::ReaderMainWidget;

mod reader_main_widget;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    pub file: Option<PathBuf>,
}

fn main() {
    env_logger::builder().init();

    let args = Args::parse();

    let filename = match args.file {
        Some(s) => s,
        None => {
            let mut last_time: Option<SystemTime> = None;
            let mut filename: Option<PathBuf> = None;

            for x in fs::read_dir("./screenshots").expect("screenshot dir absent") {
                match x {
                    Ok(dir_entry) => {
                        let p = dir_entry.path();
                        if p.extension().map(|c| c.to_str()).flatten() != Some("ron") {
                            continue;
                        };

                        let new_t = dir_entry.metadata().unwrap().modified().unwrap();
                        if last_time.map(|old_t| old_t < new_t).unwrap_or(true) {
                            last_time = Some(new_t);
                            filename = Some(dir_entry.path());
                        }
                    }
                    Err(_e) => {}
                }
            }

            filename.unwrap()
        }
    };

    let dump = match BufferOutput::from_file(filename.to_str().unwrap()) {
        Ok(d) => d,
        Err(_) => {
            std::process::exit(1);
            return;
        }
    };

    // Initializing Bernardo TUI
    App::init()
        .run_with(move |input, mut output| {
            let theme = Theme::default();
            let mut main_view = ReaderMainWidget::new(dump);

            // Genesis
            'main: loop {
                match output.clear() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("failed to clear output: {}", e);
                        break;
                    }
                }
                main_view.layout(Screenspace::new(output.size(), Rect::from_zero(output.size())));
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
                            Ok(ie) => {
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

                                main_view.act_on(ie);
                            },
                            Err(e) => {
                                error!("failed receiving input: {}", e);
                            }
                        };
                    }
                }
            }
        })
        .expect("Reader works")
}
