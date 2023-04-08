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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mode = dark_light::detect();

        match mode {
            dark_light::Mode::Dark => cc.egui_ctx.set_visuals(egui::Visuals::dark()),
            dark_light::Mode::Light => cc.egui_ctx.set_visuals(egui::Visuals::light()),
            dark_light::Mode::Default => cc.egui_ctx.set_visuals(egui::Visuals::light()),
        }

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
