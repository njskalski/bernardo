#![allow(dead_code)]
#![allow(unreachable_patterns)]

use std::io::stdout;
use std::rc::Rc;
use std::sync::Arc;

use clap::Parser;
use crossterm::terminal;
use log::{debug, error};

use bernardo::config::theme::Theme;
use bernardo::experiments::clipboard::get_me_some_clipboard;
use bernardo::fs::filesystem_front::FilesystemFront;
use bernardo::fs::real_fs::RealFS;
use bernardo::gladius::globals::{Globals, GlobalsRef};
use bernardo::gladius::load_config::load_config;
use bernardo::gladius::logger_setup::logger_setup;
use bernardo::gladius::run_gladius::run_gladius;
use bernardo::gladius::sidechannel::x::SideChannel;
use bernardo::io::crossterm_input::CrosstermInput;
use bernardo::io::crossterm_output::CrosstermOutput;
use bernardo::io::output::Output;
use bernardo::primitives::xy::XY;
use bernardo::tsw::language_set::LanguageSet;
use bernardo::tsw::tree_sitter_wrapper::TreeSitterWrapper;

// I need an option to record IO to "build" tests, not write them.

fn main() {
    let args = bernardo::gladius::args::Args::parse();
    logger_setup(args.verbosity.log_level_filter());

    #[cfg(debug_assertions)]
    {
        coredump::register_panic_handler();
    }

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
    let output = CrosstermOutput::new(stdout);

    if !output.size_constraint().visible_hint().map(|vr| vr.size > XY::ZERO).unwrap_or(false) {
        error!("it seems like the screen has zero size.");
        return;
    }

    let tree_sitter = Rc::new(TreeSitterWrapper::new(LanguageSet::full()));

    let globals: GlobalsRef = Arc::new(Globals::new(
        config_ref,
        fsf,
        clipboard,
        theme,
        tree_sitter,
    ));

    run_gladius(
        globals,
        input,
        output,
        files,
    )
}
