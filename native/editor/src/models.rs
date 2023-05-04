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
}

#[derive(Serialize, Deserialize, rustler::NifMap)]
#[allow(dead_code)]
pub(crate) struct Event {
    pub typ: Typ,
    pub data: String,
    pub serial: i64,
}

#[allow(dead_code)]
impl Event {
    pub(crate) fn new(typ: Typ, data: String, serial: i64) -> Self {
        Self { typ, data, serial }
    }

    /// Print the event to stdout.
    pub(crate) fn emit(&self) {
        // let json = format!("{}\n", serde_json::to_string(&self).unwrap());
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
