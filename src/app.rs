#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    paths: Vec<String>,
    active_file: String,
    #[serde(skip)]
    buffer: Option<String>,
    #[serde(skip)]
    output: String,
    #[serde(skip)]
    files: Vec<std::path::PathBuf>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let paths = [
            "README.md".to_string(),
            "src/main.rs".to_string(),
            "src/app.rs".to_string(),
            "src/lib.rs".to_string(),
        ];
        let files = file_list();

        Self {
            buffer: None,
            paths: paths.to_vec(),
            active_file: paths[0].to_owned(),
            output: "".to_owned(),
            files,
        }
    }
}

fn file_list() -> Vec<std::path::PathBuf> {
    std::fs::read_dir(".")
        .unwrap()
        .filter(|res| {
            res.as_ref()
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                != ".git".to_string()
        })
        .flat_map(move |res| {
            if res.as_ref().unwrap().path().is_file() {
                vec![res.unwrap().path()]
            } else {
                std::fs::read_dir(res.unwrap().path())
                    .unwrap()
                    .map(|res| res.map(|e| e.path()))
                    .filter(|path| path.as_ref().unwrap().is_file())
                    .collect::<Result<Vec<_>, std::io::Error>>()
                    .unwrap()
            }
        })
        .collect()
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    fn save_active_file(&mut self) {
        match &self.buffer {
            Some(contents) => std::fs::write(&self.active_file, contents).unwrap(),
            None => println!("no buffer to save"),
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.buffer = if let Some(contents) = &self.buffer {
            Some(contents.to_string())
        } else {
            let contents = match std::fs::read_to_string(&self.active_file.clone()) {
                Ok(contents) => contents.clone(),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    // TODO: This does not happen when a file is externally deleted
                    "read error".to_owned()
                }
            };
            Some(contents)
        };
        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            // for all self.files
            self.files.iter().for_each(|file| {
                let path = file.as_path().to_str().unwrap();
                let file_name = file.file_name().unwrap().to_str().unwrap();
                if ui.button(file_name).clicked() {
                    self.output += &(path.to_owned() + "\n");
                    self.paths.append([path.to_owned()].to_vec().as_mut());
                }
            });
        });
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for path in self.paths.clone().into_iter() {
                    let button = if path == self.active_file {
                        let button_text = egui::WidgetText::from(&path).color(egui::Color32::WHITE);
                        egui::Button::new(button_text).fill(egui::Color32::from_rgb(150, 150, 175))
                    } else {
                        egui::Button::new(&path)
                    };
                    if ui.add(button).clicked() {
                        self.save_active_file();
                        self.active_file = path;
                        match std::fs::read_to_string(&self.active_file) {
                            Ok(buffer) => self.buffer = Some(buffer),
                            Err(err) => eprintln!("Error: {}", err),
                        }
                    }
                }
            });
        });

        egui::SidePanel::right("my_right_panel").show(ctx, |ui| {
            if ui.button("Test").clicked() {
                self.output += "test\n";
            };
            if ui.button("Run").clicked() {
                self.output += "run\n";
            };
            if ui.button("Commit").clicked() {
                self.output += "commit\n";
            };
            ui.monospace(&self.output);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let mut text = match &mut self.buffer {
                    Some(buffer) => buffer,
                    None => "empty",
                }
                .to_owned();

                let text_edit = egui::TextEdit::multiline(&mut text)
                    .code_editor()
                    .desired_width(ui.available_width());
                if ui.add(text_edit).changed {
                    self.buffer = Some(text);
                    self.save_active_file();
                }
            });
            // let mut text = match &mut self.buffer {
            //     Some(buffer) => buffer,
            //     None => "empty",
            // }
            // .to_owned();
            // ui.text_edit_multiline(&mut text);
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_active_file();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
