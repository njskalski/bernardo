// #![feature(const_trait_impl)]
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
use crossterm::terminal;
use log::{debug, error, warn};

use config::theme::Theme;

use crate::experiments::clipboard::get_me_some_clipboard;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::real_fs::RealFS;
use crate::gladius::load_config::load_config;
use crate::gladius::logger_setup::logger_setup;
use crate::gladius::run_gladius::run_gladius;
use crate::io::crossterm_input::CrosstermInput;
use crate::io::crossterm_output::CrosstermOutput;
use crate::io::output::Output;
use crate::primitives::xy::XY;

#[macro_use]
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
mod lsp_client;
mod w7e;
mod promise;

// I need an option to record IO to "build" tests, not write them.


fn main() {
    let args = gladius::args::Args::parse();
    logger_setup(args.verbosity.log_level_filter());

    // Initializing subsystems
    let config_ref = load_config(args.reconfigure);
    let theme = Theme::load_or_create_default(&config_ref.config_dir).unwrap();
    let clipboard = get_me_some_clipboard();

    // Parsing arguments
    debug!("{:?}", args.paths());
    let (start_dir, files) = args.paths();
    let fsf = RealFS::new(start_dir).to_fsf();

    // Initializing Bernardo TUI
    terminal::enable_raw_mode().expect("failed entering raw mode");
    let input = CrosstermInput::new();
    let stdout = stdout();
    let mut output = CrosstermOutput::new(stdout);

    if output.size_constraint().visible_hint().size == XY::ZERO {
        error!("it seems like the screen has zero size.");
        return;
    }

    run_gladius(
        fsf,
        config_ref,
        clipboard,
        input,
        output,
        files,
        &theme,
    )
}
