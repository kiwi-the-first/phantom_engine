use crate::app::editor_app::EditorApp;

pub mod app;
pub mod egui;
pub mod menus;
pub mod panels;
pub mod workspaces;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("path: {}", args.get(1).unwrap());

    EditorApp::run().unwrap();
}
