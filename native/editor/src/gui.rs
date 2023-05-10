mod app;
mod models;

fn main() -> () {
    eframe::run_native(
        "Text Editor",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(app::EditorApp::new(cc))),
    )
    .unwrap();
    let serial_placeholder = 0;
    models::SomeEvent::new(
        models::Typ::Exit,
        vec!["byebye".to_owned()],
        serial_placeholder,
    )
    .emit();
}
