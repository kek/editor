#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    path: String,
    #[serde(skip)]
    buffer: String,
}

impl Default for EditorApp {
    fn default() -> Self {
        let path = "README.md".to_owned();
        let buffer = std::fs::read_to_string(&path).unwrap();
        Self { buffer, path: path }
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let path = ui.text_edit_singleline(&mut self.path);
            if path.changed {
                println!("path changed to {:?}", self.path);
                // self.buffer = std::fs::read_to_string(&self.path).unwrap();
                match std::fs::read_to_string(&self.path) {
                    Ok(buffer) => self.buffer = buffer,
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.buffer).code_editor(),
            );
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        std::fs::write(self.path.clone(), &self.buffer).unwrap();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
