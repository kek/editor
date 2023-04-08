fn main() -> eframe::Result<()> {
    eframe::run_native(
        "editor",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(editor::EditorApp::new(cc))),
    )
}
