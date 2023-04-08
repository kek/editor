fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Text Editor",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(editor::EditorApp::new(cc))),
    )
}
