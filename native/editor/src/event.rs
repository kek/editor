use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, rustler::NifTaggedEnum, Debug)]
pub(crate) enum EventType {
    GuiEvent,
    DebugNoBufferToSave,
    ErrorSwitchToFile,
    DebugGuiGotMessage,
    ErrorReadingFile,
    Exit,
    SetAvailableFilesCommand,
    ClickFileEvent,
    OpenFileCommand,
    SetBufferCommand,
    DebugGotUnknownMessage,
    BufferChanged,
    DebugMessage,
    NavigateUp,
}

#[derive(Serialize, Deserialize, rustler::NifMap)]
pub(crate) struct EditorEvent {
    pub typ: EventType,
    pub data: Vec<String>,
    pub serial: i64,
}

impl EditorEvent {
    pub(crate) fn new(typ: EventType, data: Vec<String>, serial: i64) -> Self {
        Self { typ, data, serial }
    }

    #[allow(dead_code)]
    pub(crate) fn emit(&self) {
        let json = serde_json::to_string(&self).unwrap();
        let stdout = &mut std::io::stdout();
        if let Err(e) = stdout.write_all(json.as_bytes()) {
            panic!("stdout closed: {}", e);
        }
        // it's either this or appending a newline to the JSON string
        if let Err(e) = stdout.flush() {
            panic!("stdout flush failed: {}", e);
        }
    }
}
