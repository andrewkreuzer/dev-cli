use std::{future::Future, path::PathBuf};

#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "javascript")]
mod javascript;
#[cfg(feature = "javascript")]
use javascript::JavaScriptLanguage;

#[cfg(feature = "lua")]
mod lua;
#[cfg(feature = "lua")]
use lua::LuaLanguage;

#[cfg(feature = "python")]
mod python;
#[cfg(feature = "python")]
use python::PythonLanguage;

mod shell;
use shell::ShellLanguage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", derive(FromPyObject))]
#[cfg_attr(feature = "python", pyo3(from_item_all))]
pub struct Dev {
    pub version: String,
    pub dir: PathBuf,
    pub steps: Vec<String>,
}

impl Dev {
    pub fn new() -> Self {
        Self {
            version: "0.1.0".to_string(),
            dir: PathBuf::new(),
            steps: Vec::new(),
        }
    }

    pub fn get_version(&self) -> String {
        self.version.clone()
    }
}

impl Default for Dev {
    fn default() -> Self {
        Self::new()
    }
}

pub trait LanguageFunctions {
    fn run_file(&self, dev: Dev, file: &str, args: Vec<&str>) -> impl Future<Output = Result<RunStatus, anyhow::Error>>;
    fn load_file(&self, file: &str) -> impl Future<Output = Result<(), anyhow::Error>>;
    fn run_shell(&self, command: &str, args: Vec<&str>) -> impl Future<Output = Result<RunStatus, anyhow::Error>>;
}

#[derive(Clone, Debug)]
pub enum Language {
    #[cfg(feature = "javascript")]
    JavaScript(JavaScriptLanguage),
    #[cfg(feature = "lua")]
    Lua(LuaLanguage),
    #[cfg(feature = "python")]
    Python(PythonLanguage),
    Shell(ShellLanguage),
    FeatureNotEnabled(String),
}

impl Language {
    pub fn get_extension(&self) -> &str {
        match self {
        Language::Python(_) => ".py",
        Language::Lua(_) => ".lua",
        #[cfg(feature = "javascript")]
        Language::JavaScript(_) => ".js",
        Language::Shell(_) => ".sh",
        _ => "", // TODO: feature not enabled??
        }
    }
}

#[derive(Debug)]
pub struct RunStatus {
    pub code: i32,
    pub message: Option<String>,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RunStatus: {}", self.code)
    }
}

#[derive(Debug)]
pub struct RunError {
    pub exit_code: Option<i32>,
    pub message: String,
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RunError: {}", self.message)
    }
}

impl std::error::Error for RunError {}

#[derive(Debug, thiserror::Error)]
pub enum LanguageError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Feature not enabled for {0}")]
    FeatureNotEnabled(String),
    #[error("Exit code: {0}")]
    ExitCode(i32),
}

impl TryFrom<&str> for Language {
    type Error = anyhow::Error;
    fn try_from(file: &str) -> Result<Self, Self::Error> {
        let extension = file.split('.').last().unwrap();
        match extension {
            #[cfg(feature = "javascript")]
            "js" => Ok(Language::JavaScript(JavaScriptLanguage::new())),
            #[cfg(feature = "lua")]
            "lua" => Ok(Language::Lua(LuaLanguage::new())),
            #[cfg(feature = "python")]
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

impl LanguageFunctions for Language {
    async fn run_file(&self, dev: Dev, file: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error> {
        match self {
            #[cfg(feature = "javascript")]
            Language::JavaScript(language) => language.run_file(dev, file).await,
            #[cfg(feature = "lua")]
            Language::Lua(language) => language.run_file(dev, file, args).await,
            #[cfg(feature = "python")]
            Language::Python(language) => language.run_file(dev, file, args).await,
            Language::Shell(language) => language.run_file(dev, file, args).await,
            Language::FeatureNotEnabled(language) => {
                Err(LanguageError::FeatureNotEnabled(language.into()).into())
            }
        }
    }

    async fn load_file(&self, file: &str) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(feature = "javascript")]
            Language::JavaScript(language) => language.load_file(file).await,
            #[cfg(feature = "lua")]
            Language::Lua(language) => language.load_file(file).await,
            #[cfg(feature = "python")]
            Language::Python(language) => language.load_file(file).await,
            Language::Shell(language) => language.load_file(file).await,
            Language::FeatureNotEnabled(language) => {
                Err(LanguageError::FeatureNotEnabled(language.into()).into())
            }
        }
    }
    async fn run_shell(&self, command: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error> {
        match self {
            #[cfg(feature = "javascript")]
            Language::JavaScript(language) => language.run_shell(command).await,
            #[cfg(feature = "lua")]
            Language::Lua(language) => language.run_shell(command, args).await,
            #[cfg(feature = "python")]
            Language::Python(language) => language.run_shell(command, args).await,
            Language::Shell(language) => language.run_shell(command, args).await,
            Language::FeatureNotEnabled(language) => {
                Err(LanguageError::FeatureNotEnabled(language.into()).into())
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
            #[cfg(feature = "javascript")]
            Language::JavaScript(_) => serializer.serialize_str("javascript"),
            #[cfg(feature = "lua")]
            Language::Lua(_) => serializer.serialize_str("lua"),
            #[cfg(feature = "python")]
            Language::Python(_) => serializer.serialize_str("python"),
            Language::Shell(_) => serializer.serialize_str("python"),
            Language::FeatureNotEnabled(language) => Err(serde::ser::Error::custom(format!(
                "Feature not enabled, enable during build with {language} feature flag"
            ))),
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
            #[cfg(feature = "javascript")]
            "javascript" | "js" => Ok(Language::JavaScript(JavaScriptLanguage::new())),
            #[cfg(feature = "lua")]
            "lua" => Ok(Language::Lua(LuaLanguage::new())),
            #[cfg(feature = "python")]
            "python" | "py" => Ok(Language::Python(PythonLanguage::new())),
            "shell" | "sh" | "bash" | "zsh" => {
                Ok(Language::Shell(ShellLanguage::new(value.as_str())))
            }
            language => match language {
                "js" | "javascript" | "lua" | "py" | "python" => {
                    Ok(Language::FeatureNotEnabled(language.into()))
                }
                _ => Err(serde::de::Error::custom(format!(
                    "Unsupported language: {language}",
                ))),
            },
        }
    }
}
