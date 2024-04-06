use log::{warn, LevelFilter};

const DEBUG_PARAMS: &[(&str, log::LevelFilter)] = &[
    // this is for git ignore
    ("globset", log::LevelFilter::Error),
    // I have no clue where it comes from, and I don't care so I suppress it
    ("mio::poll", log::LevelFilter::Error),
    ("bernardo", log::LevelFilter::Info),
    ("bernardo::primitives::tmtheme", log::LevelFilter::Info),
    ("bernardo::fs::local_filesystem_front", log::LevelFilter::Error),
    ("bernardo::gladius::run_gladius", log::LevelFilter::Info),
    ("bernardo::io::over_output", log::LevelFilter::Info),
    ("bernardo::text::buffer_state", log::LevelFilter::Warn),
    ("bernardo::tsw::tree_sitter_wrapper", log::LevelFilter::Error),
    ("bernardo::widget", log::LevelFilter::Error), // ComplexWidget lives here
    ("bernardo::widgets", log::LevelFilter::Info),
    ("bernardo::widgets::code_results_widget", log::LevelFilter::Info),
    ("bernardo::widgets::completion_widget", log::LevelFilter::Info),
    ("bernardo::widgets::dir_tree_view", log::LevelFilter::Warn),
    ("bernardo::widgets::edit_box", log::LevelFilter::Warn),
    ("bernardo::widgets::editor_widget::context_options_matrix", log::LevelFilter::Debug),
    ("bernardo::widgets::editor_widget::editor_widget", log::LevelFilter::Warn),
    ("bernardo::widgets::fuzzy_search::fuzzy_search", log::LevelFilter::Warn),
    ("bernardo::widgets::list_widget::list_widget", log::LevelFilter::Warn),
    ("bernardo::widgets::main_view::main_view", log::LevelFilter::Info),
    ("bernardo::widgets::save_file_dialog::save_file_dialog", log::LevelFilter::Warn),
    ("bernardo::widgets::with_scroll", log::LevelFilter::Warn),
    ("bernardo::layout", log::LevelFilter::Info),
    ("bernardo::layout::split_layout", log::LevelFilter::Info),
    // ("bernardo::layout::leaf_layout", log::LevelFilter::Debug),

    // This guy leaves a lot of data in trace, it seems like it spawns a new thread. I think it deserves profiling.
    ("arboard::x11_clipboard", log::LevelFilter::Warn),
    ("bernardo::w7e", log::LevelFilter::Info),
    ("bernardo::config", log::LevelFilter::Debug),
    ("bernardo::lsp_client", log::LevelFilter::Debug),
    // ("bernardo::lsp_client::lsp_read", log::LevelFilter::Warn),
    // ("bernardo::lsp_client::lsp_write", log::LevelFilter::Warn),
    ("bernardo::mocks::full_setup", log::LevelFilter::Warn),
    ("bernardo::mocks", log::LevelFilter::Warn),
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
