mod app;
mod models;
mod notification;

fn main() -> () {
    eframe::run_native(
        "Text Editor",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(app::EditorApp::new(cc))),
    )
    .unwrap();
    notification::produce("event", "exit");
}
