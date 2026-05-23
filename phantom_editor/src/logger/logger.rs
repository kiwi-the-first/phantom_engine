use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    sync::{Arc, Mutex},
};

use phantom_common::dirs::dirs;

use crate::logger::LogEntry;

pub struct Logger {
    pub buffer: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl Logger {
    pub fn create_log_file() -> Option<File> {
        let log_file = dirs::SystemDirs::cache()
            .map(|dir| {
                std::fs::create_dir_all(&dir).ok();
                dir.join("editor.log")
            })
            .and_then(|log_path| {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&log_path)
                    .ok()
            });

        log_file
    }
}
