pub struct EditorApp {
    label: String,
    buffer: String,
}

impl Default for EditorApp {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_string(),
            buffer: "lorem ipsum".to_string(),
        }
    }
}
impl EditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}
impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.label);
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.buffer).code_editor(),
            );
        });
    }
}
