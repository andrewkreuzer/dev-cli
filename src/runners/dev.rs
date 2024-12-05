use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

use crate::config::Config;

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", derive(FromPyObject))]
#[cfg_attr(feature = "python", pyo3(from_item_all))]
pub struct Dev {
    pub version: String,
    pub dir: PathBuf,
    pub steps: Vec<String>,

    environment: HashMap<String, String>,
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

    pub fn add_env(&mut self, env: (String, String)) {
        self.environment.insert(env.0, env.1);
    }

    pub fn add_envs(&mut self, envs: &HashMap<String, String>) {
        for (k, v) in envs.iter() {
            self.environment.insert(k.to_string(), v.to_string());
        }
    }
}

impl std::fmt::Display for Dev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dev(version: {}, dir: {})",
            self.version,
            self.dir.display().to_string().trim(),
        )
    }
}

impl std::fmt::Debug for Dev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nDev(
    version: {},
    dir: {},
    steps: {:?},
    environment: {:?},
)",
            self.version,
            self.dir.display().to_string().trim(),
            self.steps,
            self.environment,
        )
    }
}
