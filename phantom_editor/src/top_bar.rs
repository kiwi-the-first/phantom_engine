use egui::{CornerRadius, Ui, include_image};

use crate::{
    context::EditorContext,
    dock::DockManager,
    menus::{file::FileMenu, view::ViewMenu},
};

/// The editor's top bar: app logo, menu bar, and play/stop controls. Everything
/// above the dock area. Split into stages so the layout reads top-down instead of
/// as one deeply nested closure.
pub struct TopBar;

impl TopBar {
    pub fn show(ui: &mut Ui, editor: &mut EditorContext, dock: &mut DockManager) {
        egui::Panel::top("menu_bar").show_inside(ui, |ui| {
            ui.allocate_ui(egui::vec2(ui.available_width(), 0.0), |ui| {
                ui.horizontal_top(|ui| {
                    ui.add(
                        egui::Image::new(include_image!("images/phantom_engine_icon_glow_256.png"))
                            .fit_to_exact_size(egui::vec2(48.0, 48.0)),
                    );

                    ui.vertical(|ui| {
                        Self::menus(ui, editor, dock);
                        Self::play_controls(ui, editor);
                    });
                });
            });
        });
    }

    fn menus(ui: &mut Ui, editor: &mut EditorContext, dock: &mut DockManager) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| FileMenu::show(ui, editor));
            ui.menu_button("Edit", |_ui| {});
            ui.menu_button("Tools", |_ui| {});
            ui.menu_button("View", |ui| ViewMenu::show(ui, dock));
            ui.menu_button("Help", |_ui| {});
            ui.menu_button("Editor", |_ui| {});
        });
    }

    fn play_controls(ui: &mut Ui, editor: &mut EditorContext) {
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 40.0);
            ui.style_mut().visuals.widgets.inactive.corner_radius = CornerRadius::from(1.0);
            ui.style_mut().visuals.widgets.hovered.corner_radius = CornerRadius::from(1.0);
            ui.style_mut().visuals.widgets.active.corner_radius = CornerRadius::from(1.0);

            let play_clicked = ui.add_sized([40.0, 20.0], egui::Button::new("▶")).clicked();

            ui.add_space(-5.0);

            let stop_clicked = ui.add_sized([40.0, 20.0], egui::Button::new("■")).clicked();

            if play_clicked {
                editor.start_playing();
            }
            if stop_clicked {
                editor.stop_playing();
            }
        });
    }
}
