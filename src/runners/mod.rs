mod language;
mod dev;
mod javascript;
mod lua;
mod python;
mod shell;

pub use dev::Dev;
pub use language::{Language, LanguageFunctions};

#[derive(Debug)]
pub struct RunStatus {
    pub exit_code: Option<i32>,
    pub message: Option<String>,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (&self.exit_code, &self.message) {
            (Some(code), Some(msg)) => write!(f, "({code}) {msg}"),
            (Some(code), None) => write!(f, "({code})"),
            (None, Some(msg)) => write!(f, "{msg}"),
            _ => write!(f, "None"),
        }
    }
}

#[derive(Debug)]
pub struct RunError {
    pub exit_code: Option<i32>,
    pub message: Option<String>,
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (&self.exit_code, &self.message) {
            (Some(code), None) => write!(f, "({})RunStatus: None", code),
            (None, Some(msg)) => write!(f, "RunStatus: {}", msg),
            _ => write!(f, "RunError: None"),
        }
    }
}

impl std::error::Error for RunError {}
