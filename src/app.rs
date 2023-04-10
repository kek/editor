#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    paths: Vec<String>,
    active_file: String,
    #[serde(skip)]
    buffer: String,
}

impl Default for EditorApp {
    fn default() -> Self {
        let paths = [
            "README.md".to_owned(),
            "src/main.rs".to_owned(),
            "src/app.rs".to_owned(),
            "src/lib.rs".to_owned(),
        ]
        .to_vec();
        let active_file: String = paths[0].clone();
        let buffer = std::fs::read_to_string(&active_file).unwrap();
        Self {
            buffer,
            paths,
            active_file,
        }
    }
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        let mut zelf: EditorApp = Default::default();
        zelf.buffer = std::fs::read_to_string(&zelf.active_file).unwrap();
        zelf
    }

    fn save_active_file(&mut self) {
        let path = self.active_file.clone();

        let contents = &&self.buffer;
        println!("saving file {:?} with contents {:?}", path, contents);
        std::fs::write(path, contents).unwrap();
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for path in self.paths.clone().into_iter() {
                let file = ui.button(&path);
                if file.clicked() {
                    println!("file clicked {:?}", path);
                    self.save_active_file();
                    self.active_file = path;
                    match std::fs::read_to_string(&self.active_file) {
                        Ok(buffer) => self.buffer = buffer,
                        Err(err) => {
                            eprintln!("Error: {}", err);
                        }
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
        self.save_active_file();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
