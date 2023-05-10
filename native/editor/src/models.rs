use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, rustler::NifTaggedEnum)]
#[allow(dead_code)] // false positive because two Cargo targets?
pub(crate) enum Typ {
    GuiEvent,
    DebugNoBufferToSave,
    ErrorSwitchToFile,
    DebugGuiGotMessage,
    ErrorReadingFile,
    Error,
    Event,
    Exit,
    TestEvent,
    OpenFileCommand,
}

#[derive(Serialize, Deserialize, rustler::NifMap)]
pub(crate) struct SomeEvent {
    pub typ: Typ,
    pub data: Vec<String>,
    pub serial: i64,
}

impl SomeEvent {
    pub(crate) fn new(typ: Typ, data: Vec<String>, serial: i64) -> Self {
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
