/*
Loads config, reconfigures and exits on errors.
 */
use std::path::PathBuf;
use std::process::exit;
use std::time::SystemTime;

use log::error;

use crate::config::config::{Config, ConfigRef};
use crate::gladius::constants::{CONFIG_FILE_NAME, PROGRAM_NAME};

pub fn load_config(reconfigure: bool) -> ConfigRef {
    let config_dir_base = dirs::config_dir().unwrap_or_else(|| {
        error!("failed retrieving xdg config dir, using \"~/.config\" as default");
        PathBuf::from("~/.config")
    });
    let config_dir = config_dir_base.join(PROGRAM_NAME);
    let config_file_path = config_dir.join(CONFIG_FILE_NAME);
    let config_exists = config_file_path.exists();

    // Here we either create first config, or re-create it.
    let mut config: Option<Config> = None;
    if reconfigure || !config_exists {
        if config_exists {
            let secs = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(e) => {
                    error!("failed checking time: {}", e);
                    exit(1);
                }
            };

            let backup_path = config_dir.join(format!("{}.old.{}", CONFIG_FILE_NAME, secs));
            match std::fs::rename(&config_file_path, backup_path) {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "failed backing up config before reconfiguring: {}.\nIn order to retry, remove {:?} manually.",
                        e, &config_file_path
                    );
                    exit(2);
                }
            }
        }

        config = Some(Config::default());

        if !config_dir.exists() {
            match std::fs::create_dir_all(&config_dir) {
                Ok(_) => {}
                Err(e) => {
                    error!("failed creating config dir {:?}, due: {}", &config_dir, e);
                    exit(3);
                }
            }
        }

        match &config {
            None => {}
            Some(c) => match c.save_to_file(&config_file_path) {
                Ok(_) => {}
                Err(e) => {
                    error!("failed saving fresh config at {:?}, because {}.", &config_file_path, e);
                    exit(4);
                }
            },
        }
    }

    //loading config
    if config.is_none() {
        match Config::load_from_file(&config_file_path) {
            Ok(c) => {
                config = Some(c);
            }
            Err(e) => {
                error!("failed loading config from {:?}. because: {}", &config_file_path, e);
                exit(3);
            }
        }
    }

    let mut config = config.unwrap();
    config.config_dir = config_dir;

    let config_ref = ConfigRef::new(config);

    config_ref
}
