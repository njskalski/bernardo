use log::{LevelFilter, warn};

const DEBUG_PARAMS: &'static [(&'static str, log::LevelFilter)] = &[
    // this is for git ignore
    ("globset", log::LevelFilter::Error),
    // I have no clue where it comes from, and I don't care so I suppress it
    ("mio::poll", log::LevelFilter::Error),
    // this is for "recursive_treat_views", which is the heart and backbone of Bernardo.
    ("recursive_treat_views", log::LevelFilter::Warn),
    ("bernardo::fs::local_filesystem_front", log::LevelFilter::Error),
    ("bernardo::text::buffer_state", log::LevelFilter::Warn),
    ("bernardo::tsw::tree_sitter_wrapper", log::LevelFilter::Error),
    ("bernardo", log::LevelFilter::Debug),
    ("bernardo::widgets::main_view::main_view", log::LevelFilter::Warn),
    ("bernardo::widgets::fuzzy_search::fuzzy_search", log::LevelFilter::Warn),
    ("bernardo::widgets::edit_box", log::LevelFilter::Warn),
    ("bernardo::widgets::list_widget::list_widget", log::LevelFilter::Warn),

    // This guy leaves a lot of data in trace, it seems like it spawns a new thread. I think it deserves profiling.
    ("arboard::x11_clipboard", log::LevelFilter::Warn),
    ("bernardo::lsp_client", log::LevelFilter::Debug),
    ("bernardo::w7e", log::LevelFilter::Debug),
    ("bernardo::config", log::LevelFilter::Debug),
    ("bernardo::lsp_client::lsp_read", log::LevelFilter::Warn),
    ("bernardo::lsp_client::lsp_write", log::LevelFilter::Warn),
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
    match logger_builder.try_init() {
        Ok(_) => {}
        Err(e) => {
            warn!("failed initializing log: {:?}", e);
        }
    }
}