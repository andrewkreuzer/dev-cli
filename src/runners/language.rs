use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use super::javascript::JavaScriptLanguage;
use super::lua::LuaLanguage;
use super::python::PythonLanguage;
use super::shell::ShellLanguage;
use super::RunStatus;
use super::dev::Dev;

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
            "js" | "ts" => Ok(Language::JavaScript(JavaScriptLanguage::new())),
            "lua" => Ok(Language::Lua(LuaLanguage::new())),
            "py" => Ok(Language::Python(PythonLanguage::new())),
            "sh" | "bash" | "zsh" | "shell" => Ok(Language::Shell(ShellLanguage::new(extension))),
            language => Err(LanguageError::UnsupportedLanguage(language.into()).into()),
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
            "javascript" | "js" | "ts" => Ok(Language::JavaScript(JavaScriptLanguage::new())),
            "lua" => Ok(Language::Lua(LuaLanguage::new())),
            "python" | "py" => Ok(Language::Python(PythonLanguage::new())),
            "shell" | "sh" | "bash" | "zsh" => {
                Ok(Language::Shell(ShellLanguage::new(value.as_str())))
            }
            language => Err(serde::de::Error::custom(format!(
                "Unsupported language: {language}",
            ))),
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

