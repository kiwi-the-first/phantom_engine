use crate::app::editor_app::EditorApp;

pub mod app;
pub mod egui;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("path: {}", args.get(1).unwrap());

    EditorApp::run().unwrap();
}
