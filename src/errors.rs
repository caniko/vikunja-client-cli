use std::fmt;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CliError {
    pub ok: bool,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
}

impl CliError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: message.into(),
            code: if status > 0 { Some(status) } else { None },
        }
    }

    pub fn exit(&self) -> ! {
        let json = serde_json::to_string_pretty(self).unwrap_or_else(|e| e.to_string());
        eprintln!("{json}");
        std::process::exit(1);
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
