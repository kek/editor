use serde::{Deserialize, Serialize};

pub fn something(x: i32) -> i32 {
    x
}

#[derive(Serialize, Deserialize)]
#[allow(dead_code)] // false positive because two Cargo targets?
pub(crate) enum Typ {
    GuiEvent,
    DebugNoBufferToSave,
    ErrorSwitchToFile,
    DebugGuiGotMessage,
    ErrorReadingFile,
    Error,
    Event,
}

#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Event {
    pub typ: Typ,
    pub data: String,
}

#[allow(dead_code)]
impl Event {
    pub(crate) fn new(typ: Typ, data: String) -> Self {
        Self { typ, data }
    }

    pub(crate) fn emit(&self) {
        println!("{}", serde_json::to_string(&self).unwrap());
    }
}
