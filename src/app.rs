#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    buffer: String,
}

impl Default for EditorApp {
    fn default() -> Self {
        Self {
            buffer: "fn main() {\n\tprintln!(\"Hello World!\");\n}".to_string(),
        }
    }
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
}

impl eframe::App for EditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.buffer).code_editor(),
            );
        });
    }
}
