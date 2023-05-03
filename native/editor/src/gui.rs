mod app;
mod models;

fn main() -> () {
    eframe::run_native(
        "Text Editor",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(app::EditorApp::new(cc))),
    )
    .unwrap();
    models::Event::new(models::Typ::Exit, "byebye".to_owned()).emit();
}
