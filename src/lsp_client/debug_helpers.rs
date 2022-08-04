use std::path::{Path, PathBuf};

use log::{error, warn};
use tokio::io::AsyncWriteExt;

pub async fn lsp_debug_save(file: PathBuf, contents: String) {
    match file.parent() {
        None => {}
        Some(parent) => tokio::fs::create_dir_all(parent).await.unwrap(),
    };

    // debug!("writing to file {:?}", &file);

    let file_res = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .read(false)
        .open(file)
        .await;

    let mut file = match file_res {
        Ok(f) => f,
        Err(e) => {
            error!("failed lsp_debug_save open file, because {:?}", e);
            return;
        }
    };

    match file.write_all(contents.as_bytes()).await {
        Ok(_) => {}
        Err(e) => {
            error!("failed lsp_debug_save, because {:?}", e);
        }
    }
}

pub fn format_or_noop(s: String) -> String {
    serde_json::from_str::<serde_json::Value>(&s).ok().map(|v| serde_json::to_string_pretty(&v).ok()).flatten().unwrap_or(s)
}