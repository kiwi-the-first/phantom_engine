use crate::app::editor_app::EditorApp;

pub mod app;
pub mod egui;
pub mod panels;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("path: {}", args.get(1).unwrap());

    EditorApp::run().unwrap();
}
