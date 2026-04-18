pub struct ConsolePanel {}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("Console").show(ctx, |ui| {
            //ui here
        });
    }
}
