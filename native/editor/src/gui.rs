use event::{EditorEvent, EventType};
mod app;
mod event;

fn main() -> () {
    eframe::run_native(
        "Text Editor",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(app::EditorApp::new(cc))),
    )
    .unwrap();
    let serial = -1;
    EditorEvent::new(EventType::Exit, vec!["byebye".to_owned()], serial).emit();
}
