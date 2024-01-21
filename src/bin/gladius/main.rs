#![allow(dead_code)]
#![allow(unreachable_patterns)]

use std::sync::Arc;

use clap::Parser;
use log::debug;

use bernardo::app::App;
use bernardo::config::theme::Theme;
use bernardo::experiments::clipboard::get_me_some_clipboard;
use bernardo::fs::filesystem_front::FilesystemFront;
use bernardo::fs::real_fs::RealFS;
use bernardo::gladius::load_config::load_config;
use bernardo::gladius::logger_setup::logger_setup;
use bernardo::gladius::navcomp_loader::NavCompLoader;
use bernardo::gladius::providers::Providers;
use bernardo::gladius::real_navcomp_loader::RealNavCompLoader;
use bernardo::gladius::run_gladius::run_gladius;
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
    let fsf = RealFS::new(start_dir).unwrap().to_fsf(); // TODO unwrap

    // Initializing Bernardo TUI
    App::init().with_alt_screen_mode().run_with(move |input, output| {
        let tree_sitter = Arc::new(TreeSitterWrapper::new(LanguageSet::full()));
        let navcomp_loader = Arc::new(Box::new(RealNavCompLoader::new()) as Box<dyn NavCompLoader>);

        let providers = Providers::new(
            config_ref,
            fsf,
            clipboard,
            theme,
            tree_sitter,
            navcomp_loader,
            vec![],
        );

        run_gladius(providers, input, output, files);
    }).expect("Expected the app to work")
}
