use crate::event::{EditorEvent, EventType};

use std::fmt::format;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{self, io, thread};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EditorApp {
    pub(crate) open_files: Vec<String>,
    #[serde(skip)]
    pub(crate) active_file: Arc<Mutex<Option<String>>>,
    #[serde(skip)]
    pub(crate) buffer: Arc<Mutex<Option<String>>>,
    #[serde(skip)]
    pub(crate) output: String,
    #[serde(skip)]
    pub(crate) available_files: Arc<Mutex<Vec<String>>>,
    #[serde(skip)]
    pub(crate) outgoing_tx: mpsc::Sender<EditorEvent>,
    #[serde(skip)]
    pub(crate) incoming_rx: Arc<Mutex<mpsc::Receiver<EditorEvent>>>,
    #[serde(skip)]
    pub(crate) complete: bool,
    pub(crate) event_count: Arc<Mutex<i64>>,
    pub(crate) serial: Arc<Mutex<i64>>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let paths = [];
        let files = Arc::new(Mutex::new(file_list()));
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<EditorEvent>();
        let (incoming_tx, incoming_rx) = mpsc::channel::<EditorEvent>();

        thread::spawn(read_incoming_events(incoming_tx));
        thread::spawn(write_outgoing_events(outgoing_rx));

        Self {
            buffer: Arc::new(Mutex::new(None)),
            open_files: paths.to_vec(),
            active_file: Arc::new(Mutex::new(None)),
            output: "".to_owned(),
            available_files: files,
            outgoing_tx,
            incoming_rx: Arc::new(Mutex::new(incoming_rx)),
            complete: false,
            event_count: Arc::new(Mutex::new(0)),
            serial: Arc::new(Mutex::new(0)),
        }
    }
}

fn read_incoming_events(incoming_tx: mpsc::Sender<EditorEvent>) -> impl FnOnce() {
    move || loop {
        let mut buffer = String::new();
        {
            let this = io::stdin().read_line(&mut buffer);
            match this {
                Ok(_) => {
                    let event = {
                        match serde_json::from_str::<EditorEvent>(&buffer) {
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

fn write_outgoing_events(outgoing_rx: mpsc::Receiver<EditorEvent>) -> impl FnOnce() {
    move || loop {
        match mpsc::Receiver::recv(&outgoing_rx) {
            Ok(msg) => msg.emit(),
            Err(_) => break,
        }
    }
}

pub(crate) fn file_list() -> Vec<String> {
    [].to_vec()
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    fn listen_for_events(self: &mut EditorApp, ctx: &egui::Context) {
        if !self.complete {
            let mutex = self.incoming_rx.clone();
            self.complete = true;
            let event_count = self.event_count.clone();
            let active_file = self.active_file.clone();
            let available_files = self.available_files.clone();
            let signal = ctx.clone();
            let serial = self.serial.clone();
            let buffer = self.buffer.clone();
            let outgoing_tx = self.outgoing_tx.clone();
            thread::spawn(move || loop {
                let rx = &mutex.lock().unwrap();
                let msg = rx.recv().unwrap();
                match msg {
                    EditorEvent {
                        typ: EventType::SetAvailableFilesCommand,
                        data: paths,
                        serial: _,
                    } => {
                        *available_files.lock().unwrap() = paths;
                    }
                    EditorEvent {
                        typ: EventType::OpenFileCommand,
                        data: one_path,
                        serial: _,
                    } => {
                        let path = one_path[0].clone();
                        *active_file.lock().unwrap() = Some(path.clone());
                        match std::fs::read_to_string(path) {
                            Ok(contents) => {
                                send_event_selfless(
                                    EventType::DebugMessage,
                                    vec!["read file".to_string(), contents.clone()],
                                    &serial,
                                    &outgoing_tx,
                                );
                                *buffer.lock().unwrap() = Some(contents);
                            }
                            Err(err) => send_event_selfless(
                                EventType::ErrorReadingFile,
                                vec![err.to_string()],
                                &serial,
                                &outgoing_tx,
                            ),
                        };
                    }
                    EditorEvent {
                        typ: EventType::SetBufferCommand,
                        data: contents,
                        serial: _,
                    } => {
                        *buffer.lock().unwrap() = Some(contents[0].clone());
                    }
                    EditorEvent {
                        typ,
                        data,
                        serial: zerial,
                    } => {
                        let message = format(format_args!(
                            "{:?}, {:?}, {:?}",
                            typ,
                            data.join(", "),
                            zerial
                        ));
                        send_event_selfless(
                            EventType::DebugGotUnknownMessage,
                            vec![message],
                            &serial,
                            &outgoing_tx,
                        );
                    }
                };
                *event_count.lock().unwrap() += 1;
                signal.request_repaint();
            });
        }
    }

    fn send_event(&mut self, typ: EventType, data: Vec<String>) {
        send_event_selfless(typ, data, &self.serial, &self.outgoing_tx);
    }
}

fn send_event_selfless(
    typ: EventType,
    data: Vec<String>,
    serial: &Arc<Mutex<i64>>,
    tx: &mpsc::Sender<EditorEvent>,
) {
    let debug_event = EditorEvent::new(typ, data, *serial.lock().unwrap());
    tx.send(debug_event).unwrap();
    *serial.lock().unwrap() += 1;
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.listen_for_events(ctx);

        egui::SidePanel::left("file_list").show(ctx, |ui| {
            if ui.button("👆").clicked() {
                self.send_event(EventType::NavigateUp, vec![]);
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mutex = &self.available_files.clone();
                let files = mutex.lock().unwrap();
                files.iter().for_each(|file| {
                    let path = file;
                    let file_name = file;
                    if ui.button(file_name).clicked() {
                        self.send_event(EventType::ClickFileEvent, vec![path.to_owned()]);
                    }
                });
            });
        });

        egui::TopBottomPanel::top("file_contents").show(ctx, |ui| {
            ui.horizontal(|_ui| {
                for _path in self.open_files.clone().into_iter() {
                    // let button = match &self.active_file {
                    //     Some(active_file) => {
                    //         if path == active_file.clone() {
                    //             let button_text =
                    //                 egui::WidgetText::from(&path).color(egui::Color32::WHITE);
                    //             egui::Button::new(button_text)
                    //                 .fill(egui::Color32::from_rgb(150, 150, 175))
                    //         } else {
                    //             egui::Button::new(&path)
                    //         }
                    //     }
                    //     None => egui::Button::new(&path),
                    // };
                    // let button = ui.add(button);
                    // if button.clicked() {
                    //     self.switch_to_file(&path);
                    // }
                    // if button.clicked_by(egui::PointerButton::Secondary) {
                    //     self.open_files.retain(|p| p.to_string() != path);
                    //     if self.active_file == Some(path) {
                    //         self.active_file = None;
                    //         self.buffer = None;
                    //     }
                    // }
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
            if let Some(_path) = (&self.active_file.clone()).lock().unwrap().clone() {
                egui::ScrollArea::both().show(ui, |ui| {
                    let mut text = match self.buffer.clone().lock().unwrap().clone() {
                        Some(buffer) => buffer,
                        None => "empty".to_owned(),
                    };

                    let text_edit = egui::TextEdit::multiline(&mut text)
                        .code_editor()
                        .desired_width(ui.available_width())
                        .desired_rows(20);
                    if ui.add(text_edit).changed {
                        self.send_event(EventType::BufferChanged, vec![text.clone()]);
                        *self.buffer.lock().unwrap() = Some(text.clone());
                    }
                });
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
