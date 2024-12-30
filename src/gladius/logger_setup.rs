use std::path::PathBuf;

use flexi_logger::writers::LogWriter;
use flexi_logger::FileSpec;
use log::warn;

const DEFAULT_LEVEL: log::LevelFilter = log::LevelFilter::Info;

const DEBUG_PARAMS: &[(&str, log::LevelFilter)] = &[
    // this is for git ignore
    ("globset", log::LevelFilter::Info),
    // this is heart and mind of gladius
    ("act_on", log::LevelFilter::Info),
    // I have no clue where it comes from, and I don't care so I suppress it
    ("mio::poll", log::LevelFilter::Error),
    ("bernardo", log::LevelFilter::Info),
    ("bernardo::primitives::tmtheme", log::LevelFilter::Info),
    ("bernardo::fs::local_filesystem_front", log::LevelFilter::Error),
    ("bernardo::gladius::run_gladius", log::LevelFilter::Info),
    ("bernardo::io::over_output", log::LevelFilter::Info),
    ("bernardo::text::buffer_state", log::LevelFilter::Info),
    ("bernardo::tsw::tree_sitter_wrapper", log::LevelFilter::Error), // I have a warning I did not address there
    ("bernardo::widget", log::LevelFilter::Info),                    // ComplexWidget lives here
    ("bernardo::widgets", log::LevelFilter::Info),
    ("bernardo::widgets::code_results_widget", log::LevelFilter::Info),
    (
        "bernardo::widgets::code_results_view::stupid_symbol_usage_code_results_provider",
        log::LevelFilter::Info,
    ),
    ("bernardo::widgets::completion_widget", log::LevelFilter::Info),
    ("bernardo::widgets::dir_tree_view", log::LevelFilter::Info),
    ("bernardo::widgets::edit_box", log::LevelFilter::Info),
    ("bernardo::widgets::editor_widget::context_options_matrix", log::LevelFilter::Debug),
    ("bernardo::widgets::editor_widget::editor_widget", log::LevelFilter::Info),
    ("bernardo::widgets::fuzzy_search::fuzzy_search", log::LevelFilter::Info),
    ("bernardo::widgets::list_widget::list_widget", log::LevelFilter::Info),
    ("bernardo::widgets::main_view::main_view", log::LevelFilter::Info),
    ("bernardo::widgets::save_file_dialog::save_file_dialog", log::LevelFilter::Info),
    ("bernardo::widgets::with_scroll", log::LevelFilter::Info),
    ("bernardo::layout", log::LevelFilter::Info),
    ("bernardo::layout::split_layout", log::LevelFilter::Info),
    // ("bernardo::layout::leaf_layout", log::LevelFilter::Debug),

    // This guy leaves a lot of data in trace, it seems like it spawns a new thread. I think it deserves profiling.
    ("arboard::x11_clipboard", log::LevelFilter::Warn),
    ("bernardo::w7e", log::LevelFilter::Info),
    ("bernardo::config", log::LevelFilter::Debug),
    ("bernardo::lsp_client", log::LevelFilter::Info),
    // ("bernardo::lsp_client::lsp_read", log::LevelFilter::Warn),
    // ("bernardo::lsp_client::lsp_write", log::LevelFilter::Warn),
    ("bernardo::mocks::full_setup", log::LevelFilter::Warn),
    ("bernardo::mocks", log::LevelFilter::Warn),
];

pub fn logger_setup(stderr_on: bool, file_to_log_to: Option<PathBuf>, log_writer_op: Option<Box<dyn LogWriter>>) {
    // global logger setting
    let mut logger_builder = flexi_logger::LogSpecBuilder::new();
    logger_builder.default(DEFAULT_LEVEL);

    for (module, filter) in DEBUG_PARAMS {
        logger_builder.module(module, filter.clone());
    }

    let log_spec = logger_builder.build();

    let mut logger = flexi_logger::Logger::with(log_spec);

    let do_not_log = !stderr_on && file_to_log_to.is_none() && log_writer_op.is_none();
    if stderr_on {
        logger = logger.log_to_stderr();
    }
    if let Some(filename) = file_to_log_to {
        logger = logger.log_to_file(FileSpec::default().basename(filename.to_string_lossy()));
    }
    if let Some(log_writer) = log_writer_op {
        logger = logger.log_to_writer(log_writer);
    }

    if do_not_log {
        logger = logger.do_not_log();
    }

    match logger.start() {
        Ok(_logger) => {}
        Err(e) => {
            warn!("failed initializing log: {:?}", e);
        }
    }
}
