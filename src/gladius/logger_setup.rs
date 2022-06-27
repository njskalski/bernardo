use log::LevelFilter;

const DEBUG_PARAMS: &'static [(&'static str, log::LevelFilter)] = &[
    // this is for git ignore
    ("globset", log::LevelFilter::Error),
    // I have no clue where it comes from, and I don't care so I suppress it
    ("mio::poll", log::LevelFilter::Error),
    // this is for "recursive_treat_views", which is the heart and backbone of Bernardo.
    ("recursive_treat_views", log::LevelFilter::Error),
    ("bernardo::fs::local_filesystem_front", log::LevelFilter::Error),
    ("bernardo::text::buffer_state", log::LevelFilter::Warn),
    ("bernardo::tsw::tree_sitter_wrapper", log::LevelFilter::Error),
    ("bernardo::widgets::main_view::main_view", log::LevelFilter::Warn),
    ("bernardo::widgets::fuzzy_search::fuzzy_search", log::LevelFilter::Warn),
    ("bernardo::widgets::edit_box", log::LevelFilter::Warn),

    // This guy leaves a lot of data in trace, it seems like it spawns a new thread. I think it deserves profiling.
    ("arboard::x11_clipboard", log::LevelFilter::Warn),
];


pub fn logger_setup(level_filter: LevelFilter) {
    // global logger setting
    let mut logger_builder = env_logger::builder();

    #[cfg(not(debug_assertions))]
    logger_builder.filter_level(LevelFilter::Off);

    #[cfg(debug_assertions)]
    logger_builder.filter_level(level_filter);
    // specific logger settings
    for item in DEBUG_PARAMS {
        logger_builder.filter(Some(item.0), item.1);
    }
    logger_builder.init();
}