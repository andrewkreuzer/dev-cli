mod javascript;
mod lua;
mod python;
mod shell;

use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

use crate::config::Config;
use javascript::JavaScriptLanguage;
use lua::LuaLanguage;
use python::PythonLanguage;
use shell::ShellLanguage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", derive(FromPyObject))]
#[cfg_attr(feature = "python", pyo3(from_item_all))]
pub struct Dev {
    pub version: String,
    environment: HashMap<String, String>,
    pub dir: PathBuf,
    pub steps: Vec<String>,
}

impl Dev {
    pub fn new(config: &Config) -> Self {
        Self {
            version: "0.1.0".to_string(),
            environment: config.get_env_vars().unwrap_or(&HashMap::default()).clone(),
            dir: PathBuf::new(),
            steps: Vec::new(),
        }
    }

    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    pub fn get_dir(&self) -> PathBuf {
        self.dir.clone()
    }

    pub fn get_env(&self) -> HashMap<String, String> {
        self.environment.clone()
    }
}

#[async_trait]
#[enum_dispatch(Language)]
pub trait LanguageFunctions {
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error>;
    async fn load_file(&self, file: &str) -> Result<(), anyhow::Error>;
    async fn run_shell(&self, command: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error>;
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum Language {
    JavaScript(JavaScriptLanguage),
    Lua(LuaLanguage),
    Python(PythonLanguage),
    Shell(ShellLanguage),
}

impl Language {
    pub fn get_extension(&self) -> &str {
        match self {
            Language::Python(_) => ".py",
            Language::Lua(_) => ".lua",
            Language::JavaScript(_) => ".js",
            Language::Shell(_) => ".sh",
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = anyhow::Error;
    fn try_from(file: &str) -> Result<Self, Self::Error> {
        let extension = file.split('.').last().unwrap();
        match extension {
            "js" => Ok(Language::JavaScript(JavaScriptLanguage::new())),
            "lua" => Ok(Language::Lua(LuaLanguage::new())),
            "py" => Ok(Language::Python(PythonLanguage::new())),
            "sh" | "bash" | "zsh" | "shell" => Ok(Language::Shell(ShellLanguage::new(extension))),
            language => {
                if ["js", "javascript", "lua", "py", "python"].contains(&language) {
                    Err(LanguageError::FeatureNotEnabled(language.into()).into())
                } else {
                    Err(LanguageError::UnsupportedLanguage(language.into()).into())
                }
            }
        }
    }
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Language::JavaScript(_) => serializer.serialize_str("javascript"),
            Language::Lua(_) => serializer.serialize_str("lua"),
            Language::Python(_) => serializer.serialize_str("python"),
            Language::Shell(_) => serializer.serialize_str("python"),
        }
    }
}

impl<'a> Deserialize<'a> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Language, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "javascript" | "js" => Ok(Language::JavaScript(JavaScriptLanguage::new())),
            "lua" => Ok(Language::Lua(LuaLanguage::new())),
            "python" | "py" => Ok(Language::Python(PythonLanguage::new())),
            "shell" | "sh" | "bash" | "zsh" => {
                Ok(Language::Shell(ShellLanguage::new(value.as_str())))
            }
            language => match language {
                "js" | "javascript" | "lua" | "py" | "python" => Err(serde::de::Error::custom(
                    format!("Feature not enabled for {language}",),
                )),
                _ => Err(serde::de::Error::custom(format!(
                    "Unsupported language: {language}",
                ))),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LanguageError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Feature not enabled for {0}")]
    FeatureNotEnabled(String),
    #[error("Exit code: {0}")]
    ExitCode(i32),
}

// Probably don't need both of these
#[derive(Debug)]
pub struct RunStatus {
    pub exit_code: Option<i32>,
    pub message: Option<String>,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (&self.exit_code, &self.message) {
            (Some(code), None) => write!(f, "({})RunStatus: None", code),
            (None, Some(msg)) => write!(f, "RunStatus: {}", msg),
            _ => write!(f, "RunStatus: None"),
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
