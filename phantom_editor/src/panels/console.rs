use egui::{Color32, RichText, ScrollArea, Ui};

use crate::context::EditorContext;

pub struct ConsolePanel {}

impl ConsolePanel {
    pub fn show(ui: &mut Ui, ectx: &EditorContext) {
        let Ok(buffer) = ectx.log_buffer.lock() else {
            ui.label("Console unavailable (log buffer poisoned)");
            return;
        };

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in buffer.iter() {
                    let color = match entry.level {
                        log::Level::Error => Color32::from_rgb(255, 100, 100),
                        log::Level::Warn => Color32::from_rgb(255, 200, 100),
                        log::Level::Info => Color32::from_rgb(210, 210, 210),
                        log::Level::Debug => Color32::from_rgb(140, 160, 200),
                        log::Level::Trace => Color32::from_rgb(120, 120, 120),
                    };
                    ui.label(
                        RichText::new(format!("[{}] {}", entry.level, entry.message))
                            .color(color)
                            .monospace(),
                    );
                }
            });
    }
}
