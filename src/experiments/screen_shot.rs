use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use log::{debug, error};

use crate::io::buffer_output::BufferOutput;

pub fn screenshot(dump: &BufferOutput) {
    let SCREENSHOT_DIR: PathBuf = PathBuf::from("./screenshots/");
    fs::create_dir_all(&SCREENSHOT_DIR);

    let filename = match fs::read_dir(&SCREENSHOT_DIR) {
        Err(e) => {
            error!("failed making screenshot, read_dir: {:?}", e);
            return;
        }
        Ok(contents) => {
            let all_files = contents
                .map(|r| r.ok().map(|de| {
                    de.path()
                        .file_name()
                        .map(|c| c.to_string_lossy().to_string())
                }))
                .flatten()
                .flatten()
                .collect::<HashSet<String>>();


            let mut idx: usize = 0;
            let mut filename = format!("screenshot_{}.dump", idx);
            while all_files.contains(&filename) {
                idx += 1;
                filename = format!("screenshot_{}.dump", idx);
            }

            filename
        }
    };

    debug!("writing screenshot to file {:?}", &filename);

    match dump.save_to_file(&SCREENSHOT_DIR.join(filename)) {
        Ok(_) => {}
        Err(e) => {
            error!("failed to write screenshot");
        }
    }
}