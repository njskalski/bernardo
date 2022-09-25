use std::fs;
use std::path::PathBuf;

use log::{debug, error};

use crate::io::buffer_output::BufferOutput;
use crate::primitives::helpers::get_next_filename;

pub fn screenshot(dump: &BufferOutput) {
    let SCREENSHOT_DIR: PathBuf = PathBuf::from("./screenshots/");
    if let Err(e) = fs::create_dir_all(&SCREENSHOT_DIR) {
        error!("failed to screenshot: can't create dir: {:?}", e);
        return;
    }

    let filename = match get_next_filename(SCREENSHOT_DIR.as_path(), "screenshot_", ".ron") {
        None => {
            error!("failed to screenshot : no filename");
            return;
        }
        Some(f) => f,
    };

    debug!("writing screenshot to file {:?}", &filename);

    match dump.save_to_file(&SCREENSHOT_DIR.join(filename)) {
        Ok(_) => {}
        Err(e) => {
            error!("failed to write screenshot");
        }
    }
}