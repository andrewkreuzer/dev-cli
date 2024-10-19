use std::{future::Future, path::PathBuf};

use anyhow::Error;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    fn run_file(&self, dev: Dev, file: &str) -> impl Future<Output = Result<(), anyhow::Error>>;
    fn load_file(&self, file: &str) -> impl Future<Output = Result<(), anyhow::Error>>;
    fn run_shell(&self, command: &str) -> impl Future<Output = Result<(), anyhow::Error>>;
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

#[derive(Debug, Error)]
pub enum LanguageError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Feature not enabled for {0}")]
    FeatureNotEnabled(String),
}

impl TryFrom<&str> for Language {
    type Error = Error;
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
    async fn run_file(&self, dev: Dev, file: &str) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(feature = "javascript")]
            Language::JavaScript(language) => language.run_file(dev, file).await,
            #[cfg(feature = "lua")]
            Language::Lua(language) => language.run_file(dev, file).await,
            #[cfg(feature = "python")]
            Language::Python(language) => language.run_file(dev, file).await,
            Language::Shell(language) => language.run_file(dev, file).await,
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
    async fn run_shell(&self, command: &str) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(feature = "javascript")]
            Language::JavaScript(language) => language.run_shell(command).await,
            #[cfg(feature = "lua")]
            Language::Lua(language) => language.run_shell(command).await,
            #[cfg(feature = "python")]
            Language::Python(language) => language.run_shell(command).await,
            Language::Shell(language) => language.run_shell(command).await,
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
