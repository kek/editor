#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    paths: Vec<String>,
    #[serde(skip)]
    buffer: String,
}

impl Default for EditorApp {
    fn default() -> Self {
        let paths = ["README.md".to_owned()].to_vec();
        let buffer = std::fs::read_to_string(&paths[0]).unwrap();
        Self { buffer, paths }
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
            let path = ui.text_edit_singleline(&mut self.paths[0]);
            if path.changed {
                println!("path changed to {:?}", self.paths[0]);
                match std::fs::read_to_string(&self.paths[0]) {
                    Ok(buffer) => self.buffer = buffer,
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
            let buffer = ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.buffer).code_editor(),
            );
            if buffer.changed {
                println!("buffer changed");
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        std::fs::write(self.paths[0].clone(), &self.buffer).unwrap();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
