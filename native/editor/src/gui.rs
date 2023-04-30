use std::{
    io,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

fn main() -> () {
    eframe::run_native(
        "Text Editor",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(EditorApp::new(cc))),
    )
    .unwrap();
    message("exit");
}

fn message(msg: &str) {
    println!("{{\"event\":\"{}\"}}", msg);
}

use itertools::Itertools;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    paths: Vec<String>,
    active_file: Option<String>,
    #[serde(skip)]
    buffer: Option<String>,
    #[serde(skip)]
    output: String,
    #[serde(skip)]
    files: Vec<std::path::PathBuf>,
    #[serde(skip)]
    outgoing_tx: mpsc::Sender<String>,
    #[serde(skip)]
    incoming_rx: Arc<Mutex<Receiver<String>>>,
    #[serde(skip)]
    complete: bool,
    event_count: Arc<Mutex<i64>>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let paths = [];
        let files = file_list();
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<String>();
        let (incoming_tx, incoming_rx) = mpsc::channel::<String>();

        let tx = incoming_tx.clone();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            println!("Got message: {}", buffer);
            tx.send(buffer).unwrap();
        });
        thread::spawn(move || loop {
            let msg = Receiver::recv(&outgoing_rx).unwrap();
            println!("Made msg in gui: {}", msg);
        });

        Self {
            buffer: None,
            paths: paths.to_vec(),
            active_file: None,
            output: "".to_owned(),
            files,
            outgoing_tx,
            incoming_rx: Arc::new(Mutex::new(incoming_rx)),
            complete: false,
            event_count: Arc::new(Mutex::new(0)),
        }
    }
}

// TODO: Only goes one level deep
fn file_list() -> Vec<std::path::PathBuf> {
    std::fs::read_dir(".")
        .unwrap()
        .filter(|res| {
            // TODO: Read .gitignore
            !(path_equals(res, &".git")
                || path_equals(res, &"target")
                || path_equals(res, &"node_modules")
                || path_equals(res, &"_build")
                || path_equals(res, &".vscode")
                || path_equals(res, &".elixir_ls"))
        })
        .flat_map(|res| {
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

fn path_equals(res: &Result<std::fs::DirEntry, std::io::Error>, name: &&str) -> bool {
    res.as_ref()
        .unwrap()
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        == name.to_string()
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
            Some(contents) => std::fs::write(&self.active_file.clone().unwrap(), contents).unwrap(),
            None => println!("no buffer to save"),
        }
    }

    fn switch_to_file(&mut self, path: &String) {
        self.save_active_file();
        self.active_file = Some(path.clone());
        match std::fs::read_to_string(&self.active_file.clone().unwrap()) {
            Ok(buffer) => self.buffer = Some(buffer),
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    fn listen_for_events(&mut self, mutex: Arc<Mutex<Receiver<String>>>) {
        if !self.complete {
            self.complete = true;
            let event_count = self.event_count.clone();

            thread::spawn(move || loop {
                let rx = &mutex.lock().unwrap();
                let msg = rx.recv().unwrap();
                println!("got message in gui: {}", msg);
                *event_count.lock().unwrap() += 1;
            });
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.outgoing_tx.send("update".to_owned()).unwrap();
        let mutex = self.incoming_rx.clone();
        self.listen_for_events(mutex);

        egui::SidePanel::left("file_list").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.files.clone().iter().for_each(|file| {
                    let path = file.as_path().to_str().unwrap();
                    let file_name = file.file_name().unwrap().to_str().unwrap();
                    if ui.button(file_name).clicked() {
                        self.paths.insert(0, path.to_owned());
                        self.paths = self.paths.clone().into_iter().unique().collect();
                        self.switch_to_file(&path.to_string());
                    }
                });
            });
        });

        egui::TopBottomPanel::top("file_contents").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for path in self.paths.clone().into_iter() {
                    let button = match &self.active_file {
                        Some(active_file) => {
                            if path == active_file.clone() {
                                let button_text =
                                    egui::WidgetText::from(&path).color(egui::Color32::WHITE);
                                egui::Button::new(button_text)
                                    .fill(egui::Color32::from_rgb(150, 150, 175))
                            } else {
                                egui::Button::new(&path)
                            }
                        }
                        None => egui::Button::new(&path),
                    };
                    let button = ui.add(button);
                    if button.clicked() {
                        self.switch_to_file(&path);
                    }
                    if button.clicked_by(egui::PointerButton::Secondary) {
                        self.paths.retain(|p| p.to_string() != path);
                        if self.active_file == Some(path) {
                            self.active_file = None;
                            self.buffer = None;
                        }
                    }
                }
            });
        });

        egui::SidePanel::right("actions").show(ctx, |ui| {
            ui.label("Event count");
            ui.label(format!("{}", *self.event_count.lock().unwrap()));
            if ui.button("Test").clicked() {
                self.output += "test\n";
            };
            if ui.button("Commit").clicked() {
                self.output += "commit\n";
            };
            if ui.button("Revert").clicked() {
                self.output += "revert\n";
            };
            ui.monospace(&self.output);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.active_file {
                None => {}
                Some(path) => {
                    self.buffer = if let Some(contents) = &self.buffer {
                        Some(contents.to_string())
                    } else {
                        let contents = match std::fs::read_to_string(path) {
                            Ok(contents) => contents.clone(),
                            Err(err) => {
                                eprintln!("Error: {}", err);
                                // TODO: This does not happen when a file is
                                // externally deleted while the app is running,
                                // but it does happen when the saved state
                                // references a file which doesn't exist
                                println!("Error reading file: {}", path);
                                "read error".to_owned()
                            }
                        };
                        Some(contents)
                    };
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
                }
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_active_file();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
