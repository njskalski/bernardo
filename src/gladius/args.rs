use std::{env, fs};
use std::path::PathBuf;
use std::process::exit;
use log::error;
use clap::Parser;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity,

    #[clap(short = 'r', long = "reconfigure")]
    pub reconfigure: bool,

    #[clap(parse(from_os_str))]
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
