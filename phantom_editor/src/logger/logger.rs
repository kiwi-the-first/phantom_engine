use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
};

use log::{Level, Log};
use phantom_common::dirs::dirs;

use crate::logger::LogEntry;

pub const MAX_ENTRIES: usize = 1000;

/// Targets whose logs are dropped (matched against the record's target prefix).
///
/// `tracing::span` is the target `tracing` stamps on bridged span-lifecycle
/// records (winit 0.30 spams one per `Window::scale_factor`/`inner_size`/etc.
/// call). The span *name* — e.g. `winit::Window::scale_factor` — becomes the
/// message, so filtering by `"winit"` never matched; the target is the constant.
const NOISY_TARGETS: &[&str] = &[
    "tracing::span",
    "winit",
    "wgpu",
    "naga",
    "egui",
    "symphonia",
    "calloop",
];

pub struct PhantomLogger {
    /// Shared with the Console panel so it can render recent entries.
    pub buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    /// `None` if the log file couldn't be opened; logging still works in-memory.
    file: Mutex<Option<File>>,
    max_level: Level,
}

impl PhantomLogger {
    pub fn new(max_level: Level) -> Self {
        PhantomLogger {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_ENTRIES))),
            file: Mutex::new(Self::create_log_file()),
            max_level,
        }
    }

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

impl Log for PhantomLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if metadata.level() > self.max_level {
            return false;
        }
        let target = metadata.target();
        !NOISY_TARGETS.iter().any(|noisy| target.starts_with(noisy))
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let message = record.args().to_string();

        // Append to the log file, if we managed to open one.
        if let Ok(mut guard) = self.file.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = writeln!(file, "[{level}] {message}");
            }
        }

        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push_back(LogEntry { level, message });
            while buffer.len() > MAX_ENTRIES {
                buffer.pop_front();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut guard) = self.file.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = file.flush();
            }
        }
    }
}
