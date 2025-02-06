use std::path::PathBuf;
use std::process::exit;
use std::{env, fs};

use clap::Parser;
use log::error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// When turned on, logs are written to stderr. The verbosity of logs is hardcoded in logger_setup.rs
    #[clap(short = 'e', long = "log_to_stderr", default_value = "false")]
    pub stderr_log: bool,

    /// When set, logs are written to FILE
    #[clap(short = 'f', long = "log_to_file", default_value = None, value_name = "FILE")]
    pub file_log: Option<PathBuf>,

    /// When set, current config will be renamed to 'config.ron.old.<timestamp>' and a fresh, default config will be written to default location prior to run. Useful when a new version is released.
    #[clap(short = 'r', long = "reconfigure")]
    pub reconfigure: bool,

    #[clap(long = "record")]
    pub recording: bool,

    pub paths: Vec<PathBuf>,
}

impl Args {
    pub fn paths(&self) -> (PathBuf, Vec<PathBuf>) {
        let mut start_dir: Option<PathBuf> = None;
        let mut files_to_open: Vec<PathBuf> = Vec::default();

        for pb in self.paths.iter() {
            match fs::canonicalize(&pb) {
                Ok(p) => {
                    if p.is_dir() {
                        start_dir = Some(p);
                    } else if p.is_file() {
                        files_to_open.push(p);
                    }
                }
                Err(e) => {
                    error!("failed to canonicalize path {:?} because: {}, ignoring", &pb, e);
                }
            }
        }

        // There was a code here to find common ancestor of all files, but I am too tired to fix it.

        if start_dir.is_none() {
            start_dir = env::current_dir().ok();
        }

        if start_dir.is_none() {
            error!("failed to determine any good starting dir");
            exit(5);
        }

        (start_dir.unwrap(), files_to_open)
    }
}
