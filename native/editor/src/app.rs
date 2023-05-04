use super::models;
use itertools::Itertools;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{self, io, thread};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    pub(crate) paths: Vec<String>,
    pub(crate) active_file: Option<String>,
    #[serde(skip)]
    pub(crate) buffer: Option<String>,
    #[serde(skip)]
    pub(crate) output: String,
    #[serde(skip)]
    pub(crate) files: Vec<std::path::PathBuf>,
    #[serde(skip)]
    pub(crate) outgoing_tx: mpsc::Sender<models::Event>,
    #[serde(skip)]
    pub(crate) incoming_rx: Arc<Mutex<mpsc::Receiver<models::Event>>>,
    #[serde(skip)]
    pub(crate) complete: bool,
    pub(crate) event_count: Arc<Mutex<i64>>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let paths = [];
        let files = file_list();
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<models::Event>();
        let (incoming_tx, incoming_rx) = mpsc::channel::<models::Event>();

        thread::spawn(read_incoming_events(incoming_tx));
        thread::spawn(write_outgoing_events(outgoing_rx));

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

fn read_incoming_events(incoming_tx: mpsc::Sender<models::Event>) -> impl FnOnce() {
    move || loop {
        let mut buffer = String::new();
        {
            let this = io::stdin().read_line(&mut buffer);
            match this {
                Ok(_) => {
                    let event = {
                        match serde_json::from_str::<models::Event>(&buffer) {
                            Ok(event) => event,
                            Err(error) => {
                                panic!("error parsing JSON: «{}» in «{}»", &error, buffer)
                            }
                        }
                    };
                    incoming_tx.send(event).unwrap()
                }
                Err(err) => panic!("error reading from stdin: {}", &err),
            }
        };
    }
}

fn write_outgoing_events(outgoing_rx: mpsc::Receiver<models::Event>) -> impl FnOnce() {
    move || loop {
        match mpsc::Receiver::recv(&outgoing_rx) {
            Ok(msg) => msg.emit(),
            Err(_) => break,
        }
    }
}

// TODO: Only goes one level deep
pub(crate) fn file_list() -> Vec<std::path::PathBuf> {
    std::fs::read_dir(".")
        .unwrap()
        .filter(|res| {
            // TODO: Read .gitignore
            is_included_path(res.as_ref().unwrap())
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

fn is_included_path(direntry: &std::fs::DirEntry) -> bool {
    ![
        ".git",
        "target",
        "node_modules",
        "_build",
        ".vscode",
        ".elixir_ls",
        ".cargo",
    ]
    .iter()
    .any(|excluded| direntry_equals(direntry, excluded))
}

fn direntry_equals(res: &std::fs::DirEntry, name: &str) -> bool {
    res.path().file_name().unwrap().to_str().unwrap() == name.to_string()
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    pub(crate) fn save_active_file(&mut self) {
        match &self.buffer {
            Some(contents) => std::fs::write(&self.active_file.clone().unwrap(), contents).unwrap(),
            None => models::Event::new(
                models::Typ::DebugNoBufferToSave,
                "no buffer to save".to_owned(),
                0,
            )
            .emit(),
        }
    }

    pub(crate) fn switch_to_file(&mut self, path: &String) {
        self.save_active_file();
        self.active_file = Some(path.clone());
        match std::fs::read_to_string(&self.active_file.clone().unwrap()) {
            Ok(buffer) => self.buffer = Some(buffer),
            Err(err) => {
                let serial_placeholder = 0;
                models::Event::new(
                    models::Typ::ErrorSwitchToFile,
                    err.to_string(),
                    serial_placeholder,
                )
                .emit()
            }
        }
    }

    pub(crate) fn listen_for_events(&mut self, ctx: &egui::Context) {
        if !self.complete {
            let mutex = self.incoming_rx.clone();
            self.complete = true;
            let event_count = self.event_count.clone();
            let signal = ctx.clone();
            thread::spawn(move || loop {
                let rx = &mutex.lock().unwrap();
                let msg = rx.recv().unwrap();
                let text = serde_json::to_string(&msg).unwrap();
                let serial_placeholder = 0;
                models::Event::new(models::Typ::DebugGuiGotMessage, text, serial_placeholder)
                    .emit();
                *event_count.lock().unwrap() += 1;
                signal.request_repaint();
            });
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // How to send message from frontend to backend
        // self.outgoing_tx.send("update".to_owned()).unwrap();
        self.listen_for_events(ctx);

        egui::SidePanel::left("file_list").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.files.clone().iter().for_each(|file| {
                    let path = file.as_path().to_str().unwrap();
                    let file_name = file.file_name().unwrap().to_str().unwrap();
                    if ui.button(file_name).clicked() {
                        self.paths.insert(0, path.to_owned());
                        self.paths = self.paths.clone().into_iter().unique().collect();
                        self.switch_to_file(&path.to_string());
                        let event = models::Event::new(
                            models::Typ::GuiEvent,
                            "switch-to-file".to_owned(),
                            0,
                        );
                        self.outgoing_tx.send(event).unwrap();
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
                                let serial_placeholder = 0;
                                models::Event::new(
                                    models::Typ::Error,
                                    err.to_string(),
                                    serial_placeholder,
                                )
                                .emit();
                                // TODO: This does not happen when a file is
                                // externally deleted while the app is running,
                                // but it does happen when the saved state
                                // references a file which doesn't exist
                                models::Event::new(
                                    models::Typ::ErrorReadingFile,
                                    path.to_owned(),
                                    0,
                                )
                                .emit();
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
