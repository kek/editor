#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    paths: Vec<String>,
    active_file: String,
    #[serde(skip)]
    buffer: Option<String>,
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
        let buffer = None;
        println!("buffer: {:?}", buffer);
        println!("active file: {:?}", active_file);
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
        Default::default()
    }

    fn save_active_file(&mut self) {
        let path = self.active_file.clone();

        match self.buffer.clone() {
            Some(contents) => {
                let mid = std::cmp::min(40, contents.len());
                let summary = contents.split_at(mid).0.to_owned() + "...";
                println!("saving file {:?} with contents {:?}", path, summary);
                std::fs::write(path, contents).unwrap();
            }
            None => println!("no buffer to save"),
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.buffer = match &self.buffer {
            Some(contents) => Some(contents.to_string()),
            None => Some(match std::fs::read_to_string(&self.active_file.clone()) {
                Ok(contents) => {
                    println!("read file {:?}", contents);
                    contents.clone()
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    "read error".to_owned()
                }
            }),
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for path in self.paths.clone().into_iter() {
                    let button = if path == self.active_file {
                        let button_text = egui::WidgetText::from(&path).color(egui::Color32::WHITE);
                        egui::Button::new(button_text).fill(egui::Color32::from_rgb(150, 150, 175))
                    } else {
                        egui::Button::new(&path)
                    };
                    if ui.add(button).clicked() {
                        println!("file clicked {:?}", path);
                        self.save_active_file();
                        self.active_file = path;
                        match std::fs::read_to_string(&self.active_file) {
                            Ok(buffer) => self.buffer = Some(buffer),
                            Err(err) => {
                                eprintln!("Error: {}", err);
                            }
                        }
                    }
                }
            });

            let scroll_area = egui::ScrollArea::both();
            scroll_area.show(ui, |ui| {
                let mut text = (match &mut self.buffer {
                    Some(buffer) => buffer,
                    None => "empty",
                })
                .to_owned();
                let text_edit = egui::TextEdit::multiline(&mut text).code_editor();
                if ui.add_sized(ui.available_size(), text_edit).changed {
                    println!("buffer changed");
                    self.buffer = Some(text);
                    self.save_active_file();
                }
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_active_file();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
